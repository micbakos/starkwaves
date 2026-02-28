use crate::types::board_size::BoardSize;
use crate::types::contract::mappings::{in_game_event_keys, in_lobby_event_keys, IntoEvents};
use crate::types::contract::starkwaves::Starkwaves;
use crate::types::contract::starkwaves::Event;
use crate::types::error::GameError;
use crate::types::fire_report::FireReport;
use crate::types::game_over_outcome::GameOverOutcome;
use crate::types::game_state::{GameData, GameState, InGameState, PlayTurn};
use crate::types::result::Result;
use crate::types::{Board, Ship};
use async_trait::async_trait;
use cainome::cairo_serde::ContractAddress;
use log::info;
use starknet::core::types::{ConfirmedBlockId, L2TransactionFinalityStatus};
use starknet::{
    accounts::ConnectedAccount,
    core::types::{Felt, InvokeTransactionResult},
    providers::Provider,
};
use starknet_tokio_tungstenite::{EventSubscriptionOptions, EventsUpdate, TungsteniteStream};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use url::Url;

#[derive(Debug, Clone)]
pub enum GameUpdate {
    OpponentJoined { opponent: ContractAddress },

    ShipsPlaced,

    BoardCommitted,
    /// Game has started, contains the address of the player who attacks first
    GameStarted { your_turn: bool },
    /// You need to defend against an attack at the given position
    IncomingAttack { x: u8, y: u8 },
    /// Your attack result
    AttackResult { x: u8, y: u8, hit: bool },
    /// You were hit at the given position
    YouWereHit { x: u8, y: u8 },
    /// Game is requesting board reveal
    RevealRequested,
    /// Game has ended
    GameOver { outcome: GameOverOutcome },
}

#[async_trait]
pub trait GameCallback: Send + Sync {
    async fn on_update(&self, update: GameUpdate);
}

pub struct Game<A>
where
    A: ConnectedAccount + Sync,
{
    contract_address: Felt,
    player: A,
    ws_url: Url,
    salt: u64,
    state: GameState,
    callback: Arc<dyn GameCallback>,
    in_game_events_task: Option<JoinHandle<()>>,
    event_processor_task: Option<JoinHandle<()>>,
}

impl<A> Game<A>
where
    A: ConnectedAccount + Sync + Send + 'static,
    A::Provider: Send + Sync,
{
    pub fn player_address(&self) -> ContractAddress {
        self.player.address().into()
    }

    pub async fn join(
        contract_address: Felt,
        ws_url: Url,
        player: A,
        board_size: BoardSize,
        callback: Arc<dyn GameCallback>
    ) -> Result<Arc<Mutex<Self>>> {
        info!("Joining lobby {}", board_size);
        info!("Your Address {:#x}", player.address());

        let contract = Starkwaves::new(contract_address, &player);

        let execution = contract.request_start_game(&board_size.into());
        let result = execution.send().await.map_err(|e| e.into())?;

        let provider = player.provider();
        let events = provider
            .get_transaction_receipt(result.transaction_hash)
            .await
            .map_err(|e| e.into())
            .and_then(|info| info.receipt.into_events())?;

        let event = events
            .first()
            .expect("Expected at least one event when joining");

        match event {
            Event::PlayerEntererLobby(_) => {
                let game = Arc::new(Mutex::new(Self {
                    contract_address,
                    player,
                    ws_url: ws_url.clone(),
                    salt: rand::random(),
                    state: GameState::InLobby(board_size),
                    callback,
                    in_game_events_task: None,
                    event_processor_task: None,
                }));

                let (sender, receiver) = mpsc::channel(100);

                let event_task = tokio::spawn(async move {
                    if let Err(e) = Self::subscribe_for_lobby(
                        ws_url,
                        contract_address,
                        board_size,
                        sender,
                    ).await {
                        log::error!("Event subscription error: {}", e);
                    }
                });

                let game_clone = Arc::clone(&game);
                let processor_task = tokio::spawn(Self::process_events(game_clone, receiver));

                {
                    let mut g = game.lock().await;
                    g.in_game_events_task = Some(event_task);
                    g.event_processor_task = Some(processor_task);
                }

                Ok(game)
            }
            Event::PlayersAssembled(event) => {
                let board_size: BoardSize = board_size.into();
                let opponent = if event.player_a.0 == player.address() {
                    event.player_b.0
                } else {
                    event.player_a.0
                };
                let game_id = event.game_id;

                let game = Arc::new(Mutex::new(Self {
                    contract_address,
                    player,
                    ws_url: ws_url.clone(),
                    salt: rand::random(),
                    state: GameState::InGame(GameData::new(event.game_id, opponent.into(), board_size)),
                    callback,
                    in_game_events_task: None,
                    event_processor_task: None,
                }));

                let (sender, receiver) = mpsc::channel(100);

                let event_task = tokio::spawn(async move {
                    if let Err(e) = Self::subscribe_to_game_events(
                        ws_url,
                        contract_address,
                        game_id,
                        sender,
                    ).await {
                        log::error!("Event subscription error: {}", e);
                    }
                });

                let game_clone = Arc::clone(&game);
                let processor_task = tokio::spawn(Self::process_events(game_clone, receiver));

                {
                    let mut g = game.lock().await;
                    g.in_game_events_task = Some(event_task);
                    g.event_processor_task = Some(processor_task);
                }

                Ok(game)
            }
            _ => {
                Err(GameError::ProviderError {
                    error: format!(
                        "Expected PlayerEntererLobby or PlayersAssembled but received {:?}",
                        event
                    ),
                })
            }
        }
    }

    pub async fn place_ship(&mut self, ship: Ship) -> Result<()> {
        let callback = self.callback.clone();
        let commit_info = {
            let salt = self.salt;
            let game_data = self.in_game_data()?;
            game_data.board.place_ship(ship)?;

            if game_data.board.is_board_ready() {
                callback.on_update(GameUpdate::ShipsPlaced).await;
                let root = game_data.board.commit(salt)?;
                callback.on_update(GameUpdate::BoardCommitted).await;
                let game_id = game_data.game_id;
                Some((root, game_id))
            } else {
                None
            }
        };

        if let Some((root, game_id)) = commit_info {
            let contract = self.contract();
            let execution = contract.commit_board(&root, &game_id);
            let result: InvokeTransactionResult = execution.send().await.map_err(|e| e.into())?;

            let provider = self.player.provider();
            provider
                .get_transaction_receipt(result.transaction_hash)
                .await
                .map_err(|e| e.into())?;
        }

        Ok(())
    }

    pub async fn attack(&mut self, x: u8, y: u8) -> Result<()> {
        let player_address = self.player_address();

        let game_id = {
            let game_data = self.in_game_data()?;
            if !game_data.can_attack(&player_address) {
                return Err(GameError::CannotAttack);
            }
            game_data.game_id
        };

        let contract = self.contract();
        let execution = contract.attack(&game_id, &x, &y);
        let result: InvokeTransactionResult = execution.send().await.map_err(|e| e.into())?;

        let provider = self.player.provider();
        provider
            .get_transaction_receipt(result.transaction_hash)
            .await
            .map_err(|e| e.into())?;

        let game_data = self.in_game_data()?;
        game_data.board.track_launched_fire(x, y);
        game_data.state = InGameState::Playing(PlayTurn {
            attacking_player: player_address,
            current_attack: Some((x, y)),
        });

        Ok(())
    }

    pub async fn defend(&mut self, x: u8, y: u8) -> Result<FireReport> {
        let salt = self.salt;

        let (report, game_id) = {
            let game = self.in_game_data()?;

            game.state = InGameState::Playing(PlayTurn {
                attacking_player: game.opponent,
                current_attack: Some((x, y)),
            });

            let report = game.board.receive_fire(x, y)?;
            (report, game.game_id)
        };

        // Create contract after the mutable borrow ends
        let contract = self.contract();
        let execution = contract.defend(
            &game_id,
            &report.salted_fire_status(salt),
            &report.proof
        );
        let result: InvokeTransactionResult = execution.send().await.map_err(|e| e.into())?;

        let provider = self.player.provider();
        provider
            .get_transaction_receipt(result.transaction_hash)
            .await
            .map_err(|e| e.into())?;

        Ok(report)
    }

    pub async fn reveal(&mut self) -> Result<()> {
        let salt = self.salt;
        let (game_id, board) = {
            let game = self.in_game_data()?;
            let game_id = game.game_id;
            let board = game.board.clone();

            (game_id, board)
        };

        let contract = self.contract();
        let execution = contract.reveal(
            &game_id,
            &board.to_array()?,
            &salt.into()
        );
        let result: InvokeTransactionResult = execution.send().await.map_err(|e| e.into())?;

        let provider = self.player.provider();
        provider
            .get_transaction_receipt(result.transaction_hash)
            .await
            .map_err(|e| e.into())?;

        Ok(())
    }

    pub fn opponent(&self) -> Option<ContractAddress> {
        let in_game = self.state.as_in_game()?;

        Some(in_game.opponent)
    }

    pub fn board_size(&self) -> BoardSize {
        let state = &self.state;
        match state {
            GameState::InLobby(board_size) => *board_size,
            GameState::InGame(in_game) => in_game.board_size()
        }
    }

    pub fn board(&self) -> Result<Board> {
        let state = &self.state.as_in_game().ok_or(GameError::GameNotStarted)?;
        Ok(state.board.clone())
    }

    fn in_game_data(&mut self) -> Result<&mut GameData> {
        self.state.as_in_game_mut().ok_or(GameError::GameNotStarted)
    }

    fn contract(&self) -> Starkwaves<&A> {
        Starkwaves::new(self.contract_address, &self.player)
    }

    async fn subscribe_for_lobby(
        ws_url: Url,
        contract_address: Felt,
        board_size: BoardSize,
        sender: mpsc::Sender<Event>,
    ) -> Result<()> {
        let stream = TungsteniteStream::connect(ws_url, Duration::from_secs(5))
            .await
            .expect("WebSocket connection failed");

        let events = EventSubscriptionOptions {
            from_address: Some(contract_address),
            keys: Some(in_lobby_event_keys()),
            block_id: ConfirmedBlockId::Latest,
            finality_status: L2TransactionFinalityStatus::AcceptedOnL2,
        };

        let mut subscription = stream
            .subscribe_events(events)
            .await
            .map_err(|e| e.into())?;

        loop {
            let events_subscription = subscription.recv().await.map_err(|e| e.into())?;

            match events_subscription {
                EventsUpdate::Event(emitted_with_finality) => {
                    let emitted_event = &emitted_with_finality.emitted_event;
                    let game_event: Event = emitted_event
                        .try_into()
                        .expect("EmittedEvent should be converted to GameEvent");

                    if let Event::PlayersAssembled(event) = game_event.clone() {
                        if board_size != event.board_size.into() {
                            continue;
                        }

                        if sender.send(game_event).await.is_err() {
                            break;
                        }
                        break;
                    } else {
                        break;
                    }
                }
                EventsUpdate::Reorg(_) => {
                    break;
                }
            }
        }

        Ok(())
    }

    async fn subscribe_to_game_events(
        ws_url: Url,
        contract_address: Felt,
        game_id: Felt,
        sender: mpsc::Sender<Event>,
    ) -> Result<()> {
        let stream = TungsteniteStream::connect(ws_url, Duration::from_secs(5))
            .await
            .expect("WebSocket connection failed");

        let events = EventSubscriptionOptions {
            from_address: Some(contract_address),
            keys: Some(in_game_event_keys(game_id)),
            block_id: ConfirmedBlockId::Latest,
            finality_status: L2TransactionFinalityStatus::AcceptedOnL2,
        };

        let mut subscription = stream
            .subscribe_events(events)
            .await
            .map_err(|e| e.into())?;

        loop {
            let events_subscription = subscription.recv().await.map_err(|e| e.into())?;

            match events_subscription {
                EventsUpdate::Event(emitted_with_finality) => {
                    let emitted_event = &emitted_with_finality.emitted_event;
                    let game_event: Event = emitted_event
                        .try_into()
                        .expect("EmittedEvent should be converted to GameEvent");

                    if sender.send(game_event).await.is_err() {
                        break;
                    }
                }
                EventsUpdate::Reorg(_) => {
                    break;
                }
            }
        }

        Ok(())
    }

    fn process_events(
        game: Arc<Mutex<Self>>,
        mut rx: mpsc::Receiver<Event>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send>> {
        Box::pin(async move {
            while let Some(event) = rx.recv().await {
                if let Event::PlayersAssembled(ref assembled_event) = event {
                    let (ws_url, contract_address, opponent, board_size) = {
                        let g = game.lock().await;
                        (
                            g.ws_url.clone(),
                            g.contract_address,
                            if assembled_event.player_a.0 == g.player.address() {
                                assembled_event.player_b
                            } else {
                                assembled_event.player_a
                            },
                            assembled_event.board_size.clone(),
                        )
                    };

                    let game_id = assembled_event.game_id;

                    {
                        let mut g = game.lock().await;
                        g.state = GameState::InGame(GameData::new(
                            game_id,
                            opponent,
                            board_size.into(),
                        ));
                        g.callback.on_update(GameUpdate::OpponentJoined { opponent }).await;
                    }

                    let (sender, receiver) = mpsc::channel(100);

                    // Spawn new subscription task for in-game events
                    let event_task = tokio::spawn(async move {
                        if let Err(e) = Self::subscribe_to_game_events(
                            ws_url,
                            contract_address,
                            game_id,
                            sender,
                        ).await {
                            log::error!("Event subscription error: {}", e);
                        }
                    });

                    // Spawn new processor task
                    let game_clone = Arc::clone(&game);
                    let processor_task = tokio::spawn(Self::process_events(game_clone, receiver));

                    {
                        let mut g = game.lock().await;
                        g.in_game_events_task = Some(event_task);
                        g.event_processor_task = Some(processor_task);
                    }

                    // Exit this processor since a new one is now handling events
                    break;
                }

                let mut g = game.lock().await;
                g.handle_event(event).await;
            }
        })
    }

    async fn handle_event(&mut self, event: Event) {
        match event {
            Event::GameStarted(event) => {
                let callback = self.callback.clone();
                let in_game = self.in_game_data().unwrap();
                in_game.state = InGameState::Playing(PlayTurn {
                    attacking_player: event.attacker,
                    current_attack: None,
                });

                callback.on_update(GameUpdate::GameStarted {
                    your_turn: event.attacker == self.player_address(),
                }).await;
            }
            Event::Attack(event) => {
                if event.player != self.player_address() {
                    let callback = self.callback.clone();
                    let in_game = self.in_game_data().unwrap();
                    let turn = in_game.state.as_playing_mut().unwrap();
                    assert_eq!(event.player, in_game.opponent, "Opponent mismatch");

                    turn.current_attack = Some((event.x, event.y));
                    callback.on_update(GameUpdate::IncomingAttack {
                        x: event.x,
                        y: event.y,
                    }).await;
                    let _ = self.defend(event.x, event.x).await;
                }
            }
            Event::AttackResult(event) => {
                let callback = self.callback.clone();

                if event.attacker == self.player_address() {
                    callback.on_update(GameUpdate::AttackResult {
                        x: event.x,
                        y: event.y,
                        hit: event.ship_kind.is_some(),
                    }).await;
                } else if event.ship_kind.is_some() {
                    callback.on_update(GameUpdate::YouWereHit {
                        x: event.x,
                        y: event.y,
                    }).await;
                }

            }
            Event::GameRevealRequest(_) => {
                self.reveal().await.unwrap();

                let callback = self.callback.clone();
                callback.on_update(GameUpdate::RevealRequested).await;
            }
            Event::GameOver(event) => {
                let in_game = self.in_game_data().unwrap();
                in_game.state = InGameState::Ended;

                let callback = self.callback.clone();

                callback.on_update(GameUpdate::GameOver {
                    outcome: event.outcome.into()
                }).await;
            }
            _ => {}
        }
    }
}

impl<A> Drop for Game<A>
where
    A: ConnectedAccount + Sync,
{
    fn drop(&mut self) {
        if let Some(task) = self.in_game_events_task.take() {
            task.abort();
        }
        if let Some(task) = self.event_processor_task.take() {
            task.abort();
        }
    }
}
