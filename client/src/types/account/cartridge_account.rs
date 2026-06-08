use crate::types::account::game_account::GameAccount;
use crate::types::cartridge::cli::CartridgeCLI;
use crate::types::cartridge::types::{PolicyMethod, SessionStatus};
use crate::types::error::{CartridgeCliError, GameError};
use crate::types::result::Result;
use crate::utils::wait_success;
use async_trait::async_trait;
use starknet_rust::providers::jsonrpc::HttpTransport;
use starknet_rust::providers::JsonRpcClient;
use starknet_rust_core::chain_id;
use starknet_rust_core::types::{Call, Felt, TransactionReceiptWithBlockInfo};
use std::collections::HashSet;
use std::convert::Into;
use std::path::PathBuf;
use url::Url;

const POLICY_METHODS: [PolicyMethod; 6] = [
    PolicyMethod::new(
        "Request Start Game",
        "request_start_game",
        "Join a lobby and request to start a game for a board size",
    ),
    PolicyMethod::new(
        "Commit Board",
        "commit_board",
        "Commit the Merkle root of your ship placement",
    ),
    PolicyMethod::new(
        "Attack",
        "attack",
        "Fire at a coordinate on the opponent's board",
    ),
    PolicyMethod::new(
        "Defend",
        "defend",
        "Respond to an incoming attack with a fire-status proof",
    ),
    PolicyMethod::new(
        "Reveal",
        "reveal",
        "Reveal your ship placement at the end of the game",
    ),
    PolicyMethod::new(
        "Claim Timeout",
        "claim_timeout",
        "Claim victory if the opponent fails to act in time",
    ),
];

pub struct CartridgeAccount {
    /// Path to the `controller` binary. Defaults to `"controller"` (resolved via PATH).
    cli: CartridgeCLI,

    chain_id: Felt,
    /// The controller account address (from the registered session).
    address: Felt,
    /// The username of this account (from the registered session).
    username: String,
}


impl CartridgeAccount {

    pub async fn resolve(
        controller_path: impl Into<PathBuf>,
        contract_address: Felt,
        chain_id: Felt,
    ) -> Result<Self> {
        let cli = CartridgeCLI::new(controller_path.into(), POLICY_METHODS.to_vec());
        let status_result = cli.status().await;

        let player_address = match status_result {
            Ok(status) => {
                let validated_status = Self::status_validated(&status, contract_address).await;
                match validated_status {
                    Ok(player_address) => player_address,
                    Err(GameError::CartridgeCliError(CartridgeCliError::NoSession)) => {
                        cli.auth(contract_address, &chain_id).await?;
                        let status = cli.status().await?;
                        status.address
                    }
                    Err(error) => return Err(error),
                }
            }
            Err(GameError::CartridgeCliError(CartridgeCliError::NoSession)) => {
                cli.auth(contract_address, &chain_id).await?;
                let status = cli.status().await?;
                status.address
            }
            Err(error) => return Err(error)
        };

        let username = cli.username().await?;
        Ok(Self {
            cli,
            address: player_address,
            chain_id,
            username
        })
    }

    /// Consumes the account and wipes the local Cartridge session data.
    pub async fn logout(self) -> Result<()> {
        self.cli.clear().await
    }

    async fn status_validated(status: &SessionStatus, contract_address: Felt) -> Result<Felt> {
        Self::validate_policies(&status, contract_address)?;

        if status.is_expired {
            return Err(GameError::CartridgeCliError(CartridgeCliError::NoSession));
        }

        Ok(status.address)
    }

    fn validate_policies(status: &SessionStatus, contract_address: Felt) -> Result<()> {
        let granted: HashSet<&str> = status
            .policies
            .iter()
            .filter(|p| p.address == contract_address)
            .map(|p| p.method.as_str())
            .collect();

        let missing: Vec<&str> = POLICY_METHODS
            .iter()
            .map(|m| m.entrypoint)
            .filter(|ep| !granted.contains(ep))
            .collect();

        if missing.is_empty() {
            Ok(())
        } else {
            Err(GameError::CartridgeCliError(CartridgeCliError::NoSession))
        }
    }

    fn provider(&self) -> JsonRpcClient<HttpTransport> {
        let url = if self.chain_id == chain_id::MAINNET {
            "https://api.cartridge.gg/x/starknet/mainnet"
        } else if self.chain_id == chain_id::SEPOLIA {
            "https://api.cartridge.gg/x/starknet/sepolia"
        } else {
            // unknown chain — better to return a Result and error here
            "https://api.cartridge.gg/x/starknet/sepolia"
        };
        JsonRpcClient::new(HttpTransport::new(Url::parse(url).unwrap()))
    }
}

#[async_trait]
impl GameAccount for CartridgeAccount {
    fn address(&self) -> Felt {
        self.address
    }

    async fn send(&self, calls: Vec<Call>) -> Result<Felt> {
        self.cli.execute(calls).await
    }

    async fn send_and_wait(&self, calls: Vec<Call>) -> Result<TransactionReceiptWithBlockInfo> {
        let tx_hash = self.send(calls).await?;

        wait_success(&self.provider(), tx_hash)
            .await
            .map_err(|e| e.into())
    }
}
