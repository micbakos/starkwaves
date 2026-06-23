#[path = "common.rs"]
mod common;

use common::{CONTRACT_PATH, Config, check_sncast, wait_for_tx};
use starknet_rust::accounts::{Account, AccountError, ConnectedAccount, SingleOwnerAccount};
use starknet_rust::contract::{ContractFactory, UdcSelector};
use starknet_rust::core::types::{
    ContractExecutionError, Felt, StarknetError, TransactionExecutionErrorData,
};
use starknet_rust::providers::jsonrpc::HttpTransport;
use starknet_rust::providers::{JsonRpcClient, ProviderError};
use starknet_rust::signers::LocalWallet;
use std::fs;
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

// ── sncast ────────────────────────────────────────────────────────────────────

fn declare_contract_sncast(config: &Config) -> Result<Felt, Box<dyn std::error::Error>> {
    println!("\nDeclaring contract...");

    let mut class_hash = Felt::ZERO;
    let declare_output = Command::new("sncast")
        .current_dir(CONTRACT_PATH)
        .args([
            "--account",
            &config.account_name,
            "--wait",
            "declare",
            "--url",
            &config.rpc_url,
            "--contract-name",
            "Starkwaves",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    let mut is_already_declared = false;
    if !declare_output.stderr.is_empty() {
        let err = String::from_utf8_lossy(&declare_output.stderr);
        for line in err.lines() {
            if line.starts_with("Error:") {
                let reason = line.split(':').last().unwrap();
                if reason.contains("is already declared") {
                    is_already_declared = true;
                    break;
                }
            }
        }

        if !is_already_declared {
            eprintln!("Error: {}", err.trim());
            exit(-1);
        }
    } else {
        let out = String::from_utf8_lossy(&declare_output.stdout);
        for line in out.lines() {
            if line.starts_with("Class Hash:") {
                let hash = line.split(':').last().unwrap();
                class_hash = Felt::from_str(hash.trim())?;
                break;
            }
        }
    }

    if is_already_declared {
        println!("Contract already declared. Finding class hash...");
        let utils_output = Command::new("sncast")
            .current_dir(CONTRACT_PATH)
            .args(["utils", "class-hash", "--contract-name", "Starkwaves"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        let stdout = String::from_utf8_lossy(&utils_output.stdout);
        for line in stdout.lines() {
            if line.starts_with("Class Hash:") {
                let hash = line.split(':').last().unwrap();
                class_hash = Felt::from_str(hash.trim())?;
                break;
            }
        }

        if class_hash == Felt::ZERO {
            eprintln!("Error: Could not read class hash");
            eprintln!("{:?}", utils_output);
            exit(-1);
        }
    }

    Ok(class_hash)
}

fn deploy_contract_sncast(
    config: &Config,
    class_hash: Felt,
) -> Result<Felt, Box<dyn std::error::Error>> {
    println!("\nDeploying contract...");
    println!("Owner: {}", config.account_address);

    let class_hash_str = format!("{:#x}", class_hash);

    let mut child = Command::new("sncast")
        .current_dir(CONTRACT_PATH)
        .args([
            "--account",
            &config.account_name,
            "deploy",
            "--url",
            &config.rpc_url,
            "--class-hash",
            &class_hash_str,
            "--arguments",
            &config.account_address,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().unwrap();
    let reader = BufReader::new(stdout);

    let mut contract_address: Option<Felt> = None;

    for line in reader.lines() {
        let line = line?;
        if line.contains("Contract Address:") {
            if let Some(addr_str) = line.split_whitespace().last() {
                contract_address = Some(Felt::from_hex(addr_str)?);
            }
        }
    }

    let status = child.wait()?;

    if !status.success() {
        return Err("Deployment failed. Check the error above.".into());
    }

    contract_address.ok_or_else(|| "Could not parse contract address from sncast output".into())
}

// ── starknet-rs ───────────────────────────────────────────────────────────────

async fn declare_contract_rs(
    is_release: bool,
    account: &SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>,
) -> Result<Felt, Box<dyn std::error::Error>> {
    println!("\nDeclaring contract...");

    let (sierra, casm_class_hash) = Config::artifacts(is_release)?;
    let sierra_class_hash = sierra.class_hash()?;
    let flattened_sierra = Arc::new(sierra.flatten()?);
    let result = match account
        .declare_v3(flattened_sierra, casm_class_hash)
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
            return Ok(sierra_class_hash);
        }
        Err(e) => return Err(e.into()),
    };

    wait_for_tx(account.provider(), result.transaction_hash).await?;

    Ok(sierra_class_hash)
}

async fn deploy_contract_rs(
    class_hash: Felt,
    account: &SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>,
) -> Result<Felt, Box<dyn std::error::Error>> {
    println!("\nDeploying contract...");
    println!("Owner: {:#x}", account.address());

    let provider = account.provider();
    let factory = ContractFactory::new_with_udc(class_hash, account, UdcSelector::Legacy);
    let deployment = factory.deploy_v3(vec![account.address()], Felt::ZERO, false);
    let contract_address = deployment.deployed_address();

    let result = deployment.send().await?;
    wait_for_tx(provider, result.transaction_hash).await?;

    Ok(contract_address)
}

// ── shared ────────────────────────────────────────────────────────────────────

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

    if config.use_sncast {
        check_sncast()?;
        let class_hash = declare_contract_sncast(&config)?;
        println!("Class Hash: {:#x}", class_hash);

        let contract_address = deploy_contract_sncast(&config, class_hash)?;
        println!("Contract address: {:#x}", contract_address);

        update_env_file(&config, contract_address)?;
    } else {
        let provider = config.provider();
        let account = config.deployer_account(provider).await?;

        let class_hash = declare_contract_rs(is_release, &account).await?;
        println!("Class Hash: {:#x}", class_hash);

        let contract_address = deploy_contract_rs(class_hash, &account).await?;
        println!("Contract address: {:#x}", contract_address);

        update_env_file(&config, contract_address)?;
    }

    Ok(())
}
