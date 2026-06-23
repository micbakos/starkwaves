#![allow(warnings)]

use dotenv::dotenv;
use serde_json::Value;
use starknet_rust::accounts::{ExecutionEncoding, SingleOwnerAccount};
use starknet_rust::core::types::{
    Felt, TransactionFinalityStatus, TransactionReceiptWithBlockInfo,
};
use starknet_rust::providers::jsonrpc::HttpTransport;
use starknet_rust::providers::{JsonRpcClient, Provider};
use starknet_rust::signers::{LocalWallet, SigningKey};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;
use tokio::time::sleep;
use url::Url;

pub const CONTRACT_PATH: &str = "../contract";
const CONTRACT_FILE_NAME: &str = "starkwaves_Starkwaves";
const COMPILED_CONTRACT_SUFFIX: &str = "compiled_contract_class.json";
const SIERRA_CONTRACT_SUFFIX: &str = "contract_class.json";

#[derive(Debug)]
pub struct Config {
    env_path: PathBuf,
    pub rpc_url: String,
    pub account_name: String,
    pub account_address: String,
    pub account_private_key: Option<String>,
    pub contract_address: Option<String>,
    pub use_sncast: bool,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        dotenv().ok();
        let preset =
            env::var("PRESET").unwrap_or_else(|_| "Should have PRESET in .env".to_string());
        let env_path = PathBuf::from(format!("../.env.{}", preset));
        dotenv::from_filename(env_path.as_path()).ok();

        let rpc_url = env::var("DEPLOY_RPC_URL").map_err(|_| "DEPLOY_RPC_URL must be set")?;

        let account_name =
            env::var("DEPLOY_ACCOUNT_NAME").map_err(|_| "DEPLOY_ACCOUNT_NAME must be set")?;

        let use_sncast = env::var("USE_SNCAST")
            .map(|v| v.to_lowercase() == "true")
            .unwrap_or(false);

        let (account_address, account_private_key) = if use_sncast {
            let address = Self::get_account_address(&account_name)?;
            (address, None)
        } else {
            let (addr, pk) = Self::get_account_details(&account_name)?;
            (addr, Some(pk))
        };

        let contract_address = env::var("CONTRACT_ADDR").ok();

        println!("============================== Config ==============================");
        println!("Preset: {}", preset);
        println!(
            "Backend: {}",
            if use_sncast { "sncast" } else { "starknet-rs" }
        );
        println!("Account: {:#}", account_address);
        if let Some(contract_address) = contract_address.clone() {
            println!("Starkwaves: {:#}", contract_address);
        }
        println!("====================================================================");

        Ok(Self {
            env_path,
            rpc_url,
            account_name,
            account_address,
            account_private_key,
            contract_address,
            use_sncast,
        })
    }

    pub fn artifacts(
        is_release: bool,
    ) -> Result<(starknet_rust::core::types::contract::SierraClass, Felt), Box<dyn std::error::Error>>
    {
        let build_type = if is_release { "release" } else { "dev" };

        let directory = PathBuf::from(CONTRACT_PATH).join("target").join(build_type);
        let sierra_file_path =
            directory.join(format!("{}.{}", CONTRACT_FILE_NAME, SIERRA_CONTRACT_SUFFIX));
        let compiled_file_path = directory.join(format!(
            "{}.{}",
            CONTRACT_FILE_NAME, COMPILED_CONTRACT_SUFFIX
        ));

        println!("Sierra file: {}", sierra_file_path.to_string_lossy());
        println!("CASM file: {}", compiled_file_path.to_string_lossy());

        let sierra: starknet_rust::core::types::contract::SierraClass =
            serde_json::from_str(&fs::read_to_string(&sierra_file_path)?)?;
        let casm: starknet_rust::core::types::contract::CompiledClass =
            serde_json::from_str(&fs::read_to_string(&compiled_file_path)?)?;

        let casm_class_hash = casm.class_hash()?;
        Ok((sierra, casm_class_hash))
    }

    pub fn contract_address(&self) -> Result<&str, Box<dyn std::error::Error>> {
        self.contract_address
            .as_deref()
            .ok_or_else(|| "CONTRACT_ADDR must be set in .env".into())
    }

    pub fn env_path(&self) -> &Path {
        self.env_path.as_path()
    }

    pub fn provider(&self) -> JsonRpcClient<HttpTransport> {
        JsonRpcClient::new(HttpTransport::new(
            Url::parse(self.rpc_url.as_str()).unwrap(),
        ))
    }

    pub async fn deployer_account(
        &self,
        provider: JsonRpcClient<HttpTransport>,
    ) -> Result<
        SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>,
        Box<dyn std::error::Error>,
    > {
        let pk = self
            .account_private_key
            .as_ref()
            .ok_or("account_private_key not loaded (USE_SNCAST=true?)")?;

        let chain_id = provider.chain_id().await?;

        let signer = LocalWallet::from(SigningKey::from_secret_scalar(Felt::from_hex(pk)?));

        let account = SingleOwnerAccount::new(
            provider,
            signer,
            Felt::from_hex(self.account_address.as_str())?,
            chain_id,
            ExecutionEncoding::New,
        );

        Ok(account)
    }

    fn get_account_address(account_name: &str) -> Result<String, Box<dyn std::error::Error>> {
        let accounts_path = dirs::home_dir()
            .ok_or("Could not find home directory")?
            .join(".starknet_accounts/starknet_open_zeppelin_accounts.json");

        let content = fs::read_to_string(&accounts_path)?;
        let accounts: Value = serde_json::from_str(&content)?;

        for (_network, network_accounts) in accounts.as_object().ok_or("Invalid accounts file")? {
            if let Some(account) = network_accounts.get(account_name) {
                if let Some(address) = account.get("address").and_then(|a| a.as_str()) {
                    return Ok(address.to_string());
                }
            }
        }

        Err(format!("Account '{}' not found", account_name).into())
    }

    fn get_account_details(
        account_name: &str,
    ) -> Result<(String, String), Box<dyn std::error::Error>> {
        let accounts_path = dirs::home_dir()
            .ok_or("Could not find home directory")?
            .join(".starknet_accounts/starknet_open_zeppelin_accounts.json");

        let content = fs::read_to_string(&accounts_path)?;
        let accounts: Value = serde_json::from_str(&content)?;

        for (_network, network_accounts) in accounts.as_object().ok_or("Invalid accounts file")? {
            if let Some(account) = network_accounts.get(account_name) {
                let address = account.get("address").and_then(|a| a.as_str());
                let private_key = account.get("private_key").and_then(|a| a.as_str());
                if let Some(address) = address
                    && let Some(private_key) = private_key
                {
                    return Ok((address.to_string(), private_key.to_string()));
                }
            }
        }

        Err(format!("Account '{}' not found", account_name).into())
    }
}

pub fn check_sncast() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new("sncast").arg("--version").output();
    match output {
        Ok(o) if o.status.success() => {
            let version = String::from_utf8_lossy(&o.stdout);
            println!("Using sncast: {}", version.trim());
            Ok(())
        }
        _ => Err("sncast is not installed. Install starknet-foundry: curl -L https://raw.githubusercontent.com/foundry-rs/starknet-foundry/master/scripts/install.sh | sh".into()),
    }
}

pub async fn wait_for_tx(
    provider: &JsonRpcClient<HttpTransport>,
    tx_hash: Felt,
) -> Result<TransactionReceiptWithBlockInfo, Box<dyn std::error::Error>> {
    loop {
        match provider.get_transaction_receipt(tx_hash).await {
            Ok(receipt) => match receipt.receipt.finality_status() {
                TransactionFinalityStatus::AcceptedOnL2
                | TransactionFinalityStatus::AcceptedOnL1 => return Ok(receipt),
                _ => {}
            },
            Err(_) => {}
        }
        sleep(Duration::from_millis(500)).await;
    }
}

fn main() {
    // Does nothing
}
