#[path = "common.rs"]
mod common;

use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;
use std::process::{exit, Command, Stdio};
use std::str::FromStr;
use log::info;
use starknet::core::types::Felt;

use common::{check_sncast, Config, CONTRACT_PATH};

fn build_contract() -> Result<(), Box<dyn std::error::Error>> {
    println!("Building contract...");

    let output = Command::new("scarb")
        .current_dir(CONTRACT_PATH)
        .args(["--release", "build"])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!("Scarb build failed:\nstdout: {}\nstderr: {}", stdout, stderr).into());
    }

    println!("Contract built successfully");
    Ok(())
}

fn declare_contract(config: &Config) -> Result<Felt, Box<dyn std::error::Error>> {
    println!("\nDeclaring contract...");

    let mut class_hash = Felt::ZERO;
    let declare_output = Command::new("sncast")
        .current_dir(CONTRACT_PATH)
        .args([
            "--account", &config.account_name,
            "--wait",
            "declare",
            "--url", &config.rpc_url,
            "--contract-name", "Starkwaves",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()?;

    let mut is_already_declared = false;
    if !declare_output.stderr.is_empty() {
        let err = String::from_utf8_lossy(&declare_output.stderr);
        for line in err.lines() {
            if line.starts_with("Error:") {
                let reason = line.split(":").last().unwrap();
                if reason.contains("hash is already declared") {
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
                let hash = line.split(":").last().unwrap();
                class_hash = Felt::from_str(hash.trim())?;
                break;
            }
        }
    }

    if is_already_declared {
        println!("Contract already declared. Finding class hash...");
        let utils_output = Command::new("sncast")
            .current_dir(CONTRACT_PATH)
            .args([
                "utils",
                "class-hash",
                "--contract-name", "Starkwaves"
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()?;

        let stdout = String::from_utf8_lossy(&utils_output.stdout);
        for line in stdout.lines() {
            if line.starts_with("Class Hash:") {
                let hash = line.split(":").last().unwrap();
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

fn deploy_contract(config: &Config, class_hash: Felt, owner: &str) -> Result<Felt, Box<dyn std::error::Error>> {
    println!("\nDeploying contract...");
    println!("Owner: {}", owner);

    let class_hash_str = format!("{:#x}", class_hash);

    let mut child = Command::new("sncast")
        .current_dir(CONTRACT_PATH)
        .args([
            "--account", &config.account_name,
            "deploy",
            "--url", &config.rpc_url,
            "--class-hash", &class_hash_str,
            "--arguments", owner,
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

    let stderr = child.stderr.take().unwrap();
    let stderr_reader = BufReader::new(stderr);
    for line in stderr_reader.lines() {
        let line = line?;
        if !line.is_empty() {
            eprintln!("{}", line);
        }
    }

    if !status.success() {
        return Err("Deployment failed. Check the error above.".into());
    }

    contract_address.ok_or_else(|| "Could not parse contract address from sncast output".into())
}

fn update_env_file(
    config: &Config,
    contract_address: Felt
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    check_sncast()?;
    let config = Config::from_env()?;

    build_contract()?;
    let class_hash = declare_contract(&config)?;
    println!("Class Hash: {:#x}", class_hash);

    let contract_address = deploy_contract(&config, class_hash, &config.account_address)?;
    println!("Contract address: {:#x}", contract_address);

    update_env_file(&config, contract_address)?;

    Ok(())
}
