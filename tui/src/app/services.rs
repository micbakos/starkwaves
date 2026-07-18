use crate::app::storage::{StoredAccount, StoredAccountKind, StoredSession};
use crate::app::types::{AccountKind, LoggedAccount};
use crate::types::error::TuiError;
use crate::types::result::Result;
use starknet_rust::accounts::{ExecutionEncoding, SingleOwnerAccount};
use starknet_rust::providers::JsonRpcClient;
use starknet_rust::providers::jsonrpc::HttpTransport;
use starknet_rust::signers::{LocalWallet, SigningKey};
use starknet_rust_core::types::Felt;
use starkwaves_client::game::game::Game;
use starkwaves_client::types::account::cartridge_account::CartridgeAccount;
use starkwaves_client::types::account::game_account::GameAccount;
use starkwaves_client::types::account::local_account::LocalAccount;
use std::env;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::{Arc, RwLock};
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OnChainData {
    pub contract_address: Felt,
    pub chain_id: Felt,
    pub rpc_url: Url,
}

pub struct Services {
    pub on_chain: OnChainData,
    pub player: RwLock<Option<Arc<dyn GameAccount>>>,
    pub in_game: RwLock<Option<Game>>,
}

impl Services {
    pub fn cartridge_cli_path() -> Option<PathBuf> {
        const CLI: &str = "controller";

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

    pub async fn resolve_session(&self) -> Result<Option<LoggedAccount>> {
        let Some(storage_data) = StoredSession::read()? else {
            return Ok(None);
        };

        if storage_data.contract_address != self.on_chain.contract_address
            || storage_data.chain_id != self.on_chain.chain_id
        {
            StoredSession::delete()?;
            return Ok(None);
        }

        match storage_data.account.kind {
            StoredAccountKind::Cartridge => {
                if let Some(cli_path) = Self::cartridge_cli_path() {
                    let account = self.resolve_cartridge_account(cli_path).await?;

                    Ok(Some(account))
                } else {
                    StoredSession::delete()?;
                    Ok(None)
                }
            },
            StoredAccountKind::Env => {
                let account = self.resolve_local_account_from_env().await?;

                Ok(Some(account))
            },
        }
    }

    pub async fn remove_session(&self) -> Result<()> {
        let player = {
            let mut guard = self.player.write().unwrap();
            guard.take()
        };

        if let Some(player) = player {
            player.disconnect().await?;
        }

        StoredSession::delete()?;

        Ok(())
    }

    pub async fn resolve_cartridge_account(&self, cli_path: PathBuf) -> Result<LoggedAccount> {
        let account = CartridgeAccount::resolve(
            cli_path,
            self.on_chain.contract_address,
            self.on_chain.chain_id,
        )
        .await
        .map(|cartridge_account| {
            let logged_account = LoggedAccount {
                address: cartridge_account.address(),
                username: cartridge_account.username.clone(),
                kind: AccountKind::Cartridge,
            };

            let mut player = self.player.write().unwrap();
            *player = Some(Arc::new(cartridge_account));
            logged_account
        })?;

        StoredSession::new(
            self.on_chain.contract_address,
            self.on_chain.chain_id,
            StoredAccount {
                address: account.address,
                username: account.username.clone(),
                kind: StoredAccountKind::Cartridge,
            },
        )
        .store()?;

        Ok(account)
    }

    pub async fn resolve_local_account_from_env(&self) -> Result<LoggedAccount> {
        let address = env::var("DEV_PLAYER_ADDR")
            .map(|a| Felt::from_hex(a.as_str()).expect("Invalid DEV_PLAYER_ADDR"))
            .or(Err(TuiError::FailedToReadAccountKeysFromEnv))?;

        let private_key = env::var("DEV_PLAYER_PRIVATE_KEY")
            .map(|pk| Felt::from_hex(pk.as_str()).expect("Invalid DEV_PLAYER_PRIVATE_KEY"))
            .or(Err(TuiError::FailedToReadAccountKeysFromEnv))?;

        let signer = LocalWallet::from(SigningKey::from_secret_scalar(private_key));
        let provider = JsonRpcClient::new(HttpTransport::new(self.on_chain.rpc_url.to_owned()));
        let local_account = SingleOwnerAccount::new(
            provider,
            signer,
            address,
            self.on_chain.chain_id,
            ExecutionEncoding::New,
        );

        let mut player = self.player.write().unwrap();
        *player = Some(Arc::new(local_account));
 
        StoredSession::new(
            self.on_chain.contract_address,
            self.on_chain.chain_id,
            StoredAccount {
                address,
                username: "Local Account".into(),
                kind: StoredAccountKind::Env,
            },
        )
        .store()?;

        Ok(LoggedAccount {
            address,
            username: "Local Account".into(),
            kind: AccountKind::Local,
        })
    }
}
