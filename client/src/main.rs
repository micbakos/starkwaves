use async_trait::async_trait;
use clap::Parser;
use log::{debug, LevelFilter};
use starknet::core::types::Felt;
use starkwaves_client::game::game::{Game, GameCallback, GameUpdate};
use starkwaves_client::types::board_size::{BoardSize, SmallerBoardSize};
use starkwaves_client::types::environment::Environment;
use starkwaves_client::types::{Orientation, Ship, ShipKind};
use std::env;
use std::sync::Arc;

#[derive(Parser, Debug)]
#[command(name = "starkwaves")]
#[command(about = "Starkwaves battleship game client")]
struct Args {
    /// Player's private key (hex format, with or without 0x prefix)
    #[arg(short = 'k', long)]
    private_key: String,

    /// Player's account address (hex format, with or without 0x prefix)
    #[arg(short = 'a', long)]
    address: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    logger_init();

    let args = Args::parse();

    let private_key = Felt::from_hex(&args.private_key)
        .expect("Invalid private key format");
    let address = Felt::from_hex(&args.address)
        .expect("Invalid address format");

    let env = Environment::new();

    let rpc_provider = env.rpc_provider();
    let player = env.player(private_key, address, &rpc_provider);

    let print_callback = PrintCallback;

    // Game::join now returns Arc<Mutex<Game>> and handles event subscription internally
    let game = Game::join(
        env.contract_address,
        env.ws_url,
        player,
        BoardSize::Smaller(SmallerBoardSize::SixBySix),
        Arc::new(print_callback.clone())
    ).await?;

    {
        let game = game.lock().await;
        if let Some(opponent) = game.opponent() {
            print_callback.on_update(GameUpdate::OpponentJoined { opponent }).await;
        } else {
            println!("You entered lobby {}", game.board_size());
        }
    }


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

    Ok(())
}

#[derive(Clone)]
pub struct PrintCallback;

#[async_trait]
impl GameCallback for PrintCallback {
    async fn on_update(&self, update: GameUpdate) {
        match update {
            GameUpdate::OpponentJoined { opponent } => {
                println!("Opponent joined {:#x}. Place your ships.", opponent.0);
            }
            GameUpdate::GameStarted { first_attacker } => {
                println!("Game started! First attacker: {:#x}", first_attacker.0);
            }
            GameUpdate::IncomingAttack { x, y } => {
                println!("Incoming attack at ({}, {})", x, y);
            }
            GameUpdate::AttackResult { x, y, hit } => {
                if hit {
                    println!("Your attack at ({}, {}) was a HIT!", x, y);
                } else {
                    println!("Your attack at ({}, {}) missed.", x, y);
                }
            }
            GameUpdate::YouWereHit { x, y } => {
                println!("You were hit at ({}, {})", x, y);
            }
            GameUpdate::RevealRequested => {
                println!("Board reveal requested");
            }
            GameUpdate::GameOver { outcome } => {
                println!("Game over! Outcome: {:?}", outcome);
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