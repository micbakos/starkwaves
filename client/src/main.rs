use async_trait::async_trait;
use clap::Parser;
use log::LevelFilter;
use starkwaves_client::game::game::{Game, GameCallback, GameUpdate};
use starkwaves_client::types::board_size::{BoardSize, SmallerBoardSize};
use starkwaves_client::types::environment::Environment;
use starkwaves_client::types::game_over_outcome::{GameOverOutcome, Reason};
use starkwaves_client::types::{Orientation, Ship, ShipKind};
use std::env;
use std::process::exit;
use std::sync::Arc;
use starknet_rust::accounts::Account;
use starkwaves_client::types::contract::starkwaves::Outcome;
use starkwaves_client::types::game_state::InGameState;

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
async fn main() {
    rustls::crypto::aws_lc_rs::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");
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
    ).await.unwrap_or_else(|err| {
        eprintln!("{err}");
        exit(-1);
    });

    {
        let game = game.lock().await;
        if let Some(opponent) = game.opponent() {
            print_callback.on_update(GameUpdate::OpponentJoined { opponent }).await;
        } else {
            println!("You entered lobby {}", game.board_size());
        }
    }


    let mut input = String::new();
    let mut raw_input = Vec::new();
    loop {
        input.clear();
        raw_input.clear();
        use std::io::BufRead;
        std::io::stdin().lock().read_until(b'\n', &mut raw_input).unwrap_or_else(|err| {
            eprintln!("Failed to read prompt {err}");
            exit(-1);
        });
        input = String::from_utf8_lossy(&raw_input).into_owned();

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
                },
                ["turn"] => {
                    let game = game.lock().await;
                    let turn = game.turn()?;

                    if game.player_address() == turn.attacking_player {
                        println!("It is your turn!");
                    } else {
                        println!("It is opponent's turn!");
                    }
                },
                ["claimTimeout"] => {
                    let game = game.lock().await;

                    let outcome = game.claim_timeout().await.unwrap_or_else(|e| {
                        log::error!("Reveal failed: {e}");
                        None
                    });

                    if let Some(outcome) = outcome {
                        print_callback.on_update(GameUpdate::GameOver { outcome }).await;
                    }
                },
                ["quit"] => {
                    println!("Exiting...");
                    exit(0);
                }
                _ => {
                    println!("Commands:");
                    println!("\t- place <type> <x> <y> <h|v>");
                    println!("\t\tPosition your ship on [x, y]  horizontally (h) or vertically (v).");
                    println!("\t- attack <x> <y>");
                    println!("\t\tAttack your opponent at [x, y].");
                    println!("\t- turn");
                    println!("\t\tQuery whose turn is it.");
                    println!("\t- boards");
                    println!("\t\tDisplay your and your opponent's boards.");
                    println!("\t- claimTimeout");
                    println!("\t\tIf the opponent is taking too long to respond, you can claim timeout and win.");
                    println!("\t- quit");
                    println!("\t\tQuit the game.");
                }
            }

            Ok::<_, Box<dyn std::error::Error>>(())
        }.await {
            eprintln!("{e}");
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
            GameUpdate::AttackResult { x, y, hit, destroyed_ship } => {
                if hit {
                    println!("Your attack at ({}, {}) was a HIT!", x, y);
                    if destroyed_ship.is_some() {
                        println!("Opponent's {} was destroyed", destroyed_ship.unwrap());
                    }
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
                match outcome {
                    GameOverOutcome::Won(reason) => {
                        println!("Game over, YOU WON!");
                        if reason == Reason::FailedToProvideProof {
                            println!("The opponent failed to provide proof of their board");
                        } else if reason == Reason::TimedOut {
                            println!("The opponent failed to play in time.");
                        }
                    },
                    GameOverOutcome::Lost(reason) => {
                        match reason {
                            Reason::FairGame => println!("Game over, You Lost :("),
                            Reason::FailedToProvideProof => println!("Game over, you failed to provide proof of your board"),
                            Reason::TimedOut => {
                                println!("Game over, you failed to play in time.");
                            }
                        }
                    }
                }
                exit(0);
            }
            GameUpdate::Reset => {
                println!("Starkwaves has to stop. Game owner reset the state of the contract.");
                exit(0);
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