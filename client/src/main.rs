use async_trait::async_trait;
use clap::Parser;
use log::LevelFilter;
use starkwaves_client::game::game::{Game, GameCallback, GameUpdate};
use starkwaves_client::types::board_size::{BoardSize, SmallerBoardSize};
use starkwaves_client::types::environment::Environment;
use starkwaves_client::types::{Orientation, Ship, ShipKind};
use std::env;
use std::process::exit;
use std::sync::Arc;


#[derive(Parser, Debug)]
#[command(name = "starkwaves")]
#[command(about = "Starkwaves battleship game client")]
struct Args {
    /// Player's private key (hex format, with or without 0x prefix)
    #[arg(short = 'k', long)]
    private_key: Option<String>,

    /// Player's account address (hex format, with or without 0x prefix)
    #[arg(short = 'a', long)]
    address: Option<String>,

    /// Use a hardcoded preset player: A or B
    #[arg(short = 'p', long)]
    preset: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    logger_init();

    let args = Args::parse();

    let env = Environment::new();

    let rpc_provider = env.rpc_provider();

    let player = env.player(
        args.preset.as_deref(),
        args.private_key.as_deref(),
        args.address.as_deref(),
        &rpc_provider,
    );

    let print_callback = PrintCallback;

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

        if let Err(e) = async {
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
                ["boards"] => {
                    let game = game.lock().await;
                    let board = game.board()?;
                    println!("{}", board.launched_fire_view());
                    println!();
                    println!("{}", board);
                }
                ["quit"] => {
                    println!("Exiting...");
                    exit(0);
                }
                _ => {
                    println!("Commands:");
                    println!("\t- place <type> <x> <y> <h|v>");
                    println!("\t- attack <x> <y>");
                    println!("\t- boards");
                    println!("\t- quit");
                }
            }

            Ok::<_, Box<dyn std::error::Error>>(())
        }.await {
            eprintln!("Error: {}", e);
        }
    }
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
            GameUpdate::ShipsPlaced => {
                println!("Ships placed. Committing board...");
            }
            GameUpdate::BoardCommitted => {
                println!("BoardCommitted.");
            }
            GameUpdate::GameStarted { your_turn } => {
                if your_turn {
                    println!("Game started! Your turn to attack.");
                } else {
                    println!("Game started! Opponent's turn to attack.");
                }
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