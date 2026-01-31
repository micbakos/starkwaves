use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use dotenv::dotenv;
use serde_json::Value;

pub const CONTRACT_PATH: &str = "../contract";

#[derive(Debug)]
pub struct Config {
    env_path: PathBuf,
    pub rpc_url: String,
    pub account_name: String,
    pub account_address: String,
    pub contract_address: Option<String>,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        dotenv().ok();
        let preset = env::var("PRESET").unwrap_or_else(|_| "Should have PRESET in .env".to_string());
        let env_path = PathBuf::from(format!("../.env.{}", preset));
        dotenv::from_filename(env_path.as_path()).ok();

        let rpc_url = env::var("DEPLOY_RPC_URL")
            .map_err(|_| "DEPLOY_RPC_URL must be set")?;

        let account_name = env::var("DEPLOY_ACCOUNT_NAME")
            .map_err(|_| "DEPLOY_ACCOUNT_NAME must be set")?;

        let account_address = Self::get_account_address(&account_name)?;

        let contract_address = env::var("CONTRACT_ADDR").ok();

        println!("============================== Config ==============================");
        println!("Preset: {}", preset);
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
            contract_address,
        })
    }

    pub fn contract_address(&self) -> Result<&str, Box<dyn std::error::Error>> {
        self.contract_address
            .as_deref()
            .ok_or_else(|| "CONTRACT_ADDR must be set in .env".into())
    }

    pub fn env_path(&self) -> &Path {
        self.env_path.as_path()
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


fn main() {
    // Does nothing
}