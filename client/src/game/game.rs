use crate::game::event_handler::EventHandler;
use crate::types::board_size::BoardSize;
use crate::types::contract::args::{AttackArgs, CommitBoardArgs, DefendArgs, RevealArgs, StartGameArgs};
use crate::types::contract::events::GameEvent;
use crate::types::error::{CodecError, GameError};
use crate::types::fire_report::FireReport;
use crate::types::game_state::GameState;
use crate::types::result::Result;
use crate::types::{Board, Ship};
use starknet::accounts::Account;
use starknet::core::types::{ConfirmedBlockId, L2TransactionFinalityStatus, TransactionReceipt};
use starknet::{
    accounts::ConnectedAccount,
    core::{
        types::{Call, Felt, InvokeTransactionResult},
        utils::get_selector_from_name,
    },
    providers::Provider,
};
use starknet_tokio_tungstenite::{EventSubscriptionOptions, EventsUpdate, TungsteniteStream};
use std::sync::Arc;
use std::time::Duration;
use log::info;
use tokio::sync::Mutex;
use url::Url;

pub struct Game<A>
where
    A: ConnectedAccount + Sync,
{
    contract_address: Felt,
    player: A,
    opponent: Felt,
    pub game_id: Felt,
    board: Board,
    salt: u64,
    state: GameState
}

impl<A> Game<A>
where
    A: ConnectedAccount + Sync,
    A::Provider: Send + Sync,
{
    pub fn player_address(&self) -> Felt {
        self.player.address()
    }

    pub fn opponent_address(&self) -> Felt {
        self.opponent
    }

    pub async fn create(
        contract_address: Felt,
        host: A,
        opponent: Felt,
        board_size: BoardSize,
    ) -> Result<Self> {
        info!("Creating new game {}", board_size);
        info!("Host {:#x}", host.address());
        info!("Opponent {:#x}", opponent);
        let selector = get_selector_from_name("start_game")
            .expect("Invalid selector");

        let star_game_args = StartGameArgs::new(opponent, board_size);
        let call = Call {
            to: contract_address,
            selector,
            calldata: star_game_args.try_into()
                .map_err(|e: CodecError| e.into())?,
        };

        let execution = host.execute_v3(vec![call]);
        let result: InvokeTransactionResult = execution.send().await.map_err(|e| e.into())?;

        let provider = host.provider();
        let event: GameEvent = provider
            .get_transaction_receipt(result.transaction_hash)
            .await
            .map(|info| {
                let event = if let TransactionReceipt::Invoke(invoke) = info.receipt {
                    invoke.events.first().cloned().unwrap()
                } else {
                    panic!(
                        "Should receive invoke transaction, instead got {:?}",
                        info.receipt
                    );
                };

                TryFrom::try_from(event).expect("PlayersAssembled event not emitted")
            })
            .map_err(|e| e.into())?;

        let players_assembled = event
            .as_players_assembled()
            .expect("Event emitted should be PlayersAssembled");

        info!("Players assembled, ships can be placed...");

        Ok(Self {
            contract_address,
            player: host,
            opponent,
            game_id: *players_assembled.0,
            board: Board::new(board_size),
            salt: rand::random(),
            state: GameState::PlacingShips
        })
    }

    pub async fn place_ship(
        &mut self,
        ship: Ship
    ) -> Result<()> {
        self.board.place_ship(ship)?;

        if self.board.is_board_ready() {
            let root = self.board.commit(self.salt)?;

            let selector = get_selector_from_name("commit_board")
                .expect("Invalid selector");
            let commit_board_args = CommitBoardArgs {
                root,
                game_id: self.game_id,
            };

            let call = Call {
                to: self.contract_address,
                selector,
                calldata: commit_board_args.try_into().map_err(|e: CodecError| e.into())?,
            };

            let execution = self.player.execute_v3(vec![call]);
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
        if !self.can_attack() {
            return Err(GameError::CannotAttack);
        }

        let selector = get_selector_from_name("attack")
            .expect("Invalid selector");
        let attack_args = AttackArgs {
            game_id: self.game_id,
            x,
            y,
        };

        let call = Call {
            to: self.contract_address,
            selector,
            calldata: attack_args.try_into().map_err(|e: CodecError| e.into())?,
        };

        let execution = self.player.execute_v3(vec![call]);
        let result: InvokeTransactionResult = execution.send().await.map_err(|e| e.into())?;

        let provider = self.player.provider();
        provider
            .get_transaction_receipt(result.transaction_hash)
            .await
            .map_err(|e| e.into())?;

        self.state = GameState::Playing {
            attacking_player: self.player_address(),
            current_attack: Some((x, y)),
        };

        Ok(())
    }

    pub async fn defend(
        &mut self,
        x: u8,
        y: u8,
    ) -> Result<FireReport> {
        self.state = GameState::Playing {
            attacking_player: self.opponent,
            current_attack: Some((x, y)),
        };

        let report = self.board.receive_fire(x, y)?;

        let selector = get_selector_from_name("defend")
            .expect("Invalid selector");

        let args = DefendArgs::new(
            self.game_id,
            &report,
            self.salt,
        );

        let call = Call {
            to: self.contract_address,
            selector,
            calldata: args.try_into().map_err(|e: CodecError| e.into())?,
        };

        let execution = self.player.execute_v3(vec![call]);
        let result: InvokeTransactionResult = execution.send().await.map_err(|e| e.into())?;

        let provider = self.player.provider();
        provider
            .get_transaction_receipt(result.transaction_hash)
            .await
            .map_err(|e| e.into())?;

        Ok(report)
    }

    pub async fn reveal(
        &mut self
    ) -> Result<()> {
        let selector = get_selector_from_name("reveal")
            .expect("Invalid selector");

        let args = RevealArgs {
            game_id: self.game_id,
            board: self.board.to_array()?,
            salt: self.salt.into()
        };

        let call = Call {
            to: self.contract_address,
            selector,
            calldata: args.try_into().map_err(|e: CodecError| e.into())?,
        };

        let execution = self.player.execute_v3(vec![call]);
        let result: InvokeTransactionResult = execution.send().await.map_err(|e| e.into())?;

        let provider = self.player.provider();
        provider
            .get_transaction_receipt(result.transaction_hash)
            .await
            .map_err(|e| e.into())?;

        Ok(())
    }

    pub async fn subscribe_to_events<EH>(
        &mut self,
        ws_url: Url,
        handler: Arc<Mutex<EH>>,
    ) -> Result<()>
    where
        EH: EventHandler + 'static,
    {
        let stream = TungsteniteStream::connect(ws_url, Duration::from_secs(5))
            .await
            .expect("WebSocket connection failed");

        let events = EventSubscriptionOptions {
            from_address: Some(self.contract_address),
            keys: Some(GameEvent::keys(self.game_id)),
            block_id: ConfirmedBlockId::Latest,
            finality_status: L2TransactionFinalityStatus::AcceptedOnL2,
        };

        let mut subscription = stream.subscribe_events(events).await.map_err(|e| e.into())?;

        loop {
            let h = handler.lock().await;

            let events_subscription = subscription.recv().await.map_err(|e| e.into())?;

            match events_subscription {
                EventsUpdate::Event(emitted_with_finality) => {
                    let game_event: GameEvent = emitted_with_finality.emitted_event.try_into()
                        .expect("EmmittedEvent event should be converted to GameEvent");

                    h.handle_event(game_event.clone()).await;

                    if game_event.is_game_over() {
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

    pub fn on_game_started(&mut self, attacker: Felt) {
        self.state = GameState::Playing {
            attacking_player: attacker,
            current_attack: None,
        }
    }

    pub fn can_attack(&self) -> bool {
        if let GameState::Playing { attacking_player, current_attack } = &self.state {
            self.player_address() == *attacking_player && current_attack.is_none()
        } else {
            false
        }
    }
}
