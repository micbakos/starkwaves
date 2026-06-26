use crate::app::types::{AccountKind, LoggedAccount};
use starknet_rust_core::types::Felt;
use starkwaves_client::game::game::Game;
use starkwaves_client::types::account::cartridge_account::CartridgeAccount;
use starkwaves_client::types::account::game_account::GameAccount;
use starkwaves_client::types::result::Result;
use std::env;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::RwLock;
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnChainData {
    pub contract_address: Felt,
    pub chain_id: Felt,
    pub rpc_url: Url,
}

pub struct Services {
    pub on_chain: OnChainData,
    pub player: RwLock<Option<Box<dyn GameAccount>>>,
    pub in_game: RwLock<Option<Game>>,
}

impl Services {
    pub fn cartridge_cli_path() -> Option<PathBuf> {
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

    pub fn new(on_chain_data: OnChainData) -> Self {
        Self {
            on_chain: on_chain_data,
            player: RwLock::new(None),
            in_game: RwLock::new(None),
        }
    }

    pub async fn resolve_cartridge_account(&self, cli_path: PathBuf) -> Result<LoggedAccount> {
        CartridgeAccount::resolve(
            cli_path,
            self.on_chain.contract_address,
            self.on_chain.chain_id,
        )
        .await
        .map(|cartridge_account| {
            let logged_account = LoggedAccount {
                address: cartridge_account.address(),
                kind: AccountKind::Cartridge,
            };

            let mut player = self.player.write().unwrap();
            *player = Some(Box::new(cartridge_account));
            logged_account
        })
    }
}
