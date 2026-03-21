#[path = "common.rs"]
mod common;

use crate::common::wait_for_tx;
use common::{CONTRACT_PATH, Config, check_sncast};
use starknet::accounts::{Account, AccountError, ConnectedAccount, SingleOwnerAccount};
use starknet::contract::{ContractFactory, UdcSelector};
use starknet::core::types::contract::SierraClass;
use starknet::core::types::{
    ContractExecutionError, Felt, StarknetError, TransactionExecutionErrorData,
};
use starknet::providers::jsonrpc::HttpTransport;
use starknet::providers::{JsonRpcClient, Provider, ProviderError};
use starknet::signers::LocalWallet;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio, exit};
use std::str::FromStr;
use std::sync::Arc;

fn build_contract(is_release: bool) -> Result<(), Box<dyn std::error::Error>> {
    println!("Building contract...");

    let mut args = vec![];
    if is_release {
        args.push("--release");
    }
    args.push("build");

    let output = Command::new("scarb")
        .current_dir(CONTRACT_PATH)
        .args(&args)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!(
            "Scarb build failed:\nstdout: {}\nstderr: {}",
            stdout, stderr
        )
        .into());
    }

    println!("Contract built successfully");
    Ok(())
}

async fn declare_contract(
    is_release: bool,
    account: &SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>,
) -> Result<Felt, Box<dyn std::error::Error>> {
    println!("\nDeclaring contract...");

    let (sierra, casm) = Config::artifacts(is_release)?;
    let class_hash = sierra.class_hash()?;
    let flattened_sierra = Arc::new(sierra.flatten()?);
    let result = match account
        .declare_v3(flattened_sierra, casm.class_hash()?)
        .send()
        .await
    {
        Ok(r) => r,
        Err(AccountError::Provider(ProviderError::StarknetError(
            StarknetError::TransactionExecutionError(TransactionExecutionErrorData {
                execution_error: ContractExecutionError::Message(ref msg),
                ..
            }),
        ))) if msg.contains("already declared") => {
            println!("Contract already declared.");
            return Ok(class_hash);
        }
        Err(e) => return Err(e.into()),
    };

    wait_for_tx(account.provider(), result.transaction_hash).await?;

    Ok(class_hash)
}

async fn deploy_contract(
    class_hash: Felt,
    account: &SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>,
) -> Result<Felt, Box<dyn std::error::Error>> {
    println!("\nDeploying contract...");
    println!("Owner: {:#x}", account.address());

    let factory = ContractFactory::new_with_udc(class_hash, account, UdcSelector::Legacy);
    let deployment = factory.deploy_v3(vec![account.address()], Felt::ZERO, false);
    let contract_address = deployment.deployed_address();

    let result = deployment.send().await?;
    wait_for_tx(account.provider(), result.transaction_hash).await?;

    Ok(contract_address)
}

fn update_env_file(
    config: &Config,
    contract_address: Felt,
) -> Result<(), Box<dyn std::error::Error>> {
    let env_path = config.env_path();

    let content = if env_path.exists() {
        fs::read_to_string(env_path)?
    } else {
        String::new()
    };

    let contract_addr_line = format!("CONTRACT_ADDR={:#x}", contract_address);
    let mut lines: Vec<String> = content.lines().map(String::from).collect();

    let mut found = false;
    for line in &mut lines {
        if line.starts_with("CONTRACT_ADDR=") {
            *line = contract_addr_line.clone();
            found = true;
            break;
        }
    }

    if !found {
        lines.push(contract_addr_line);
    }

    let mut file = fs::File::create(env_path)?;
    for line in lines {
        writeln!(file, "{}", line)?;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_env()?;

    let is_release = true;
    build_contract(is_release)?;
    let provider = config.provider();
    let account = config.deployer_account(provider).await?;

    let class_hash = declare_contract(is_release, &account).await?;
    println!("Class Hash: {:#x}", class_hash);

    let contract_address = deploy_contract(class_hash, &account).await?;
    println!("Contract address: {:#x}", contract_address);

    update_env_file(&config, contract_address)?;

    Ok(())
}
