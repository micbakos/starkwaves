use std::env;
use async_trait::async_trait;
use starknet::accounts::ConnectedAccount;
use starkwaves_client::game::event_handler::EventHandler;
use starkwaves_client::game::game::Game;
use starkwaves_client::types::board_size::{BoardSize, SmallerBoardSize};
use starkwaves_client::types::contract::events::GameEvent;
use starkwaves_client::types::environment::Environment;
use starkwaves_client::types::{Orientation, Ship, ShipKind};
use std::sync::Arc;
use log::{debug, LevelFilter};
use tokio::sync::Mutex;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    logger_init();

    let env = Environment::new();
    debug!("{:?}", env);

    let rpc_provider = env.rpc_provider();
    let host = env.host(&rpc_provider);

    let game = Game::create(
        env.contract_address,
        host.clone(),
        env.opponent(),
        BoardSize::Smaller(SmallerBoardSize::SixBySix)
    ).await?;

    let game = Arc::new(Mutex::new(game));
    let handler = StarkwavesHandler::new(Arc::clone(&game));
    let handler = Arc::new(Mutex::new(handler));

    let game_for_subscription = Arc::clone(&game);
    let handler_clone = Arc::clone(&handler);
    let events_task = tokio::spawn(async move {
        let mut game = game_for_subscription.lock().await;

        if let Err(e) = game.subscribe_to_events(
            env.ws_url,
            handler_clone
        ).await {
            eprintln!("Event subscription error: {}", e);
        }
    });

    let mut input = String::new();
    loop {
        input.clear();
        std::io::stdin().read_line(&mut input)?;

        let parts: Vec<&str> = input.trim().split_whitespace().collect();

        match parts.as_slice() {
            ["place", ship_type, x, y, orientation] => {
                let kind = ShipKind::try_from(*ship_type)?;
                let x: u8 = x.parse()?;
                let y: u8 = y.parse()?;
                let orientation = Orientation::try_from(*orientation)?;

                let ship = Ship::new(kind, x, y, orientation);

                let mut game = game.lock().await;
                game.place_ship(ship).await?;
            }
            ["attack", x, y] => {
                let x: u8 = x.parse()?;
                let y: u8 = y.parse()?;

                let mut game = game.lock().await;
                game.attack(x, y).await?;
            }
            ["quit"] => {
                println!("Exiting...");
                break;
            }
            _ => {
                println!("Commands:");
                println!("\t- place <type> <x> <y> <h|v>");
                println!("\t- attack <x> <y>");
                println!("\t- quit");
            }
        }
    }

    events_task.abort();

    Ok(())
}

struct StarkwavesHandler<A>
where
    A: ConnectedAccount + Sync + Send,
{
    game: Arc<Mutex<Game<A>>>,
}

impl<A> StarkwavesHandler<A>
where
    A: ConnectedAccount + Sync + Send,
    A::Provider: Send + Sync,
{
    fn new(game: Arc<Mutex<Game<A>>>) -> Self {
        Self { game }
    }
}

#[async_trait]
impl<A> EventHandler for StarkwavesHandler<A>
where
    A: ConnectedAccount + Sync + Send + 'static,
    A::Provider: Send + Sync,
{
    async fn handle_event(&self, event: GameEvent) {
        match event {
            GameEvent::PlayersAssembled { .. } => {
                // Do nothing, already handled
            }
            GameEvent::GameStarted { attacker, .. } => {
                let mut game = self.game.lock().await;
                game.on_game_started(attacker);

                let player_address = game.player_address();
                if player_address == attacker {
                    println!("Match started. It is your turn...")
                } else {
                    println!("Match started. Wait for the opponent to fire first...")
                }
            }
            GameEvent::Attack { player, x, y, .. } => {
                let mut game = self.game.lock().await;

                if player == game.player_address() {
                    // Ignore, we got a report about the attack we just did.
                } else if player == game.opponent_address() {
                    let x: u8 = x.try_into().unwrap();
                    let y: u8 = y.try_into().unwrap();
                    let _ = game.defend(x, y).await;
                }
            }
            GameEvent::Hit { attacker, x, y, ship_kind, .. } => {
                let game = self.game.lock().await;
                if attacker == game.player_address() {
                    println!("HIT!");
                    println!("- ({}, {}) => {}", x, y, ship_kind)
                } else if attacker == game.opponent_address() {
                    println!("You got bombed");
                    println!("- ({}, {}) => {}", x, y, ship_kind)
                }
            }
            GameEvent::GameRevealRequest { .. } => {
                println!("GameReveal {:?}", event);
            }
            GameEvent::GameOver { .. } => {
                println!("GameOver {:?}", event);
            }
        }
    }
}

pub fn logger_init() {
    let mut builder = colog::basic_builder();
    builder.filter(None, LevelFilter::Debug);
    if let Ok(rust_log) = env::var("RUST_LOG") {
        builder.parse_filters(&rust_log);
    }
    builder.init();
}