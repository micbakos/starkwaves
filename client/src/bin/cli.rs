use async_trait::async_trait;
use clap::Parser;
use dotenv::dotenv;
use log::LevelFilter;
use starknet_rust::accounts::{ExecutionEncoding, SingleOwnerAccount};
use starknet_rust::providers::JsonRpcClient;
use starknet_rust::providers::jsonrpc::HttpTransport;
use starknet_rust::signers::{LocalWallet, SigningKey};
use starknet_rust_core::types::Felt;
use starknet_rust_core::utils::cairo_short_string_to_felt;
use starkwaves_client::game::game::{Game, GameCallback, GameUpdate};
use starkwaves_client::types::account::cartridge_account::CartridgeAccount;
use starkwaves_client::types::account::game_account::GameAccount;
use starkwaves_client::types::board_size::{BoardSize, SmallerBoardSize};
use starkwaves_client::types::game_over_outcome::{GameOverOutcome, Reason};
use starkwaves_client::types::result::Result;
use starkwaves_client::types::{Orientation, Ship, ShipKind};
use std::env;
use std::path::PathBuf;
use std::process::{Command, Stdio, exit};
use std::sync::Arc;
use tokio::sync::mpsc;
use url::Url;

#[derive(Parser, Debug)]
#[command(name = "starkwaves")]
#[command(about = "Starkwaves battleship game client")]
pub struct Args {
    /// Player's private key (hex format, with or without 0x prefix)
    #[arg(short = 'k', long)]
    private_key: Option<String>,

    /// Player's account address (hex format, with or without 0x prefix)
    #[arg(short = 'a', long)]
    address: Option<String>,
}

impl Args {
    pub fn local_key(&self) -> Option<LocalKey> {
        if self.private_key.is_some() && self.address.is_some() {
            Some(LocalKey {
                private_key: Felt::from_hex(self.private_key.clone().unwrap().as_str()).unwrap(),
                address: Felt::from_hex(self.address.clone().unwrap().as_str()).unwrap(),
            })
        } else if self.address.is_none() && self.private_key.is_none() {
            None
        } else {
            panic!("-a (address) and -k (private_key) should be specified together");
        }
    }
}

#[derive(Debug, Clone)]
pub enum PlayerPreset {
    Local(LocalKey),
    Cartridge(PathBuf),
}

#[derive(Debug, Clone)]
pub struct LocalKey {
    pub private_key: Felt,
    pub address: Felt,
}

#[derive(Debug, Clone)]
pub struct Environment {
    rpc_url: Url,
    pub ws_url: Url,
    chain_id: Felt,
    pub contract_address: Felt,
    preset: PlayerPreset,
}

impl Environment {
    pub fn new(args: &Args) -> Self {
        dotenv().ok();
        let preset =
            env::var("PRESET").unwrap_or_else(|_| "Should have PRESET in .env".to_string());
        dotenv::from_filename(format!(".env.{}", preset)).ok();

        let chain_id_str = env::var("CHAIN_ID").expect("Should have CHAIN_ID in .env");
        let rpc_url_str = env::var("RPC_URL").expect("Should have RPC_URL in .env");
        let ws_url_str = env::var("WS_URL").expect("Should have WS_URL in .env");

        let contract_address_str = env::var("CONTRACT_ADDR")
            .expect("Should have CONTRACT_ADDRESS in .env.\nRun: \n\tcargo run --bin deploy");
        let contract_address =
            Felt::from_hex(contract_address_str.as_str()).expect("Invalid contract address");

        let local_key = args.local_key();
        let preset = match local_key {
            None => {
                let controller_cli_path = Self::controller_cli_path();
                if controller_cli_path.is_none() {
                    panic!(
                        "Cartridge controller cli should be installed if not private key and address is provided\n ↳ https://github.com/cartridge-gg/controller-cli#installation"
                    );
                }

                PlayerPreset::Cartridge(controller_cli_path.unwrap())
            }
            Some(local_key) => PlayerPreset::Local(local_key),
        };

        Self {
            rpc_url: Url::parse(rpc_url_str.as_str()).expect("Invalid RPC_URL"),
            ws_url: Url::parse(ws_url_str.as_str()).expect("Invalid WS_URL"),
            chain_id: cairo_short_string_to_felt(chain_id_str.as_str()).expect("Invalid CHAIN_ID"),
            contract_address,
            preset,
        }
    }

    fn controller_cli_path() -> Option<PathBuf> {
        const CLI: &str = "controller";

        // 1. On PATH? (`Command::new` searches PATH on unix.)
        let on_path = Command::new(CLI)
            .arg("--version")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false);
        if on_path {
            return Some(PathBuf::from(CLI));
        }

        // 2. Custom INSTALL_DIR from install.sh, then 3. its default.
        let candidates = [
            env::var_os("INSTALL_DIR").map(PathBuf::from),
            dirs::home_dir().map(|h| h.join(".local/bin")),
        ];
        candidates
            .into_iter()
            .flatten()
            .map(|dir| dir.join(CLI))
            .find(|p| p.exists())
    }

    pub fn rpc_provider(&self) -> JsonRpcClient<HttpTransport> {
        JsonRpcClient::new(HttpTransport::new(self.rpc_url.to_owned()))
    }

    pub async fn player(&self, rpc: &JsonRpcClient<HttpTransport>) -> Result<Arc<dyn GameAccount>> {
        match &self.preset {
            PlayerPreset::Local(local_key) => {
                let signer =
                    LocalWallet::from(SigningKey::from_secret_scalar(local_key.private_key));

                Ok(Arc::new(SingleOwnerAccount::new(
                    rpc.clone(),
                    signer,
                    local_key.address,
                    self.chain_id,
                    ExecutionEncoding::New,
                )))
            }
            PlayerPreset::Cartridge(cli_path) => {
                let cartridge_account =
                    CartridgeAccount::resolve(cli_path, self.contract_address, self.chain_id)
                        .await?;

                Ok(Arc::new(cartridge_account))
            }
        }
    }
}

#[tokio::main]
async fn main() {
    starkwaves_client::install_crypto_provider();
    logger_init();

    let args = Args::parse();

    let env = Environment::new(&args);

    let rpc_provider = env.rpc_provider();

    let player = env.player(&rpc_provider).await.unwrap_or_else(|err| {
        eprintln!("{err}");
        exit(-1);
    });

    let (updates_sender, mut updates_receiver) = mpsc::unbounded_channel::<GameUpdate>();

    tokio::spawn(async move {
        while let Some(update) = updates_receiver.recv().await {
            let _ = PrintCallback.on_update(update);
        }
    });

    let game = Game::join(
        env.contract_address,
        env.ws_url,
        player,
        BoardSize::Smaller(SmallerBoardSize::SixBySix),
        updates_sender,
    )
    .await
    .unwrap_or_else(|err| {
        eprintln!("{err}");
        exit(-1);
    });

    {
        let game = game.lock().await;
        if let Some(opponent) = game.opponent() {
            PrintCallback
                .on_update(GameUpdate::OpponentJoined { opponent })
                .await;
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
        std::io::stdin()
            .lock()
            .read_until(b'\n', &mut raw_input)
            .unwrap_or_else(|err| {
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

#[derive(Clone, Copy)]
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
            GameUpdate::AttackResult {
                x,
                y,
                hit,
                destroyed_ship,
            } => {
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
                    }
                    GameOverOutcome::Lost(reason) => match reason {
                        Reason::FairGame => println!("Game over, You Lost :("),
                        Reason::FailedToProvideProof => {
                            println!("Game over, you failed to provide proof of your board")
                        }
                        Reason::TimedOut => {
                            println!("Game over, you failed to play in time.");
                        }
                    },
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
