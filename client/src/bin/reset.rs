#[path = "common.rs"]
mod common;

use common::{CONTRACT_PATH, Config, check_sncast, wait_for_tx};
use log::error;
use starknet_rust::accounts::{Account, ConnectedAccount, SingleOwnerAccount};
use starknet_rust::core::types::{BlockId, BlockTag, Call, Felt, FunctionCall};
use starknet_rust::core::utils::get_selector_from_name;
use starknet_rust::providers::jsonrpc::HttpTransport;
use starknet_rust::providers::{JsonRpcClient, Provider};
use starknet_rust::signers::LocalWallet;
use std::env;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};

// ── sncast ────────────────────────────────────────────────────────────────────

fn reset_game_sncast(config: &Config, game_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let contract_address = config.contract_address()?;

    println!("Resetting game {}...", game_id);

    let mut child = Command::new("sncast")
        .current_dir(CONTRACT_PATH)
        .args([
            "--account",
            &config.account_name,
            "--wait",
            "invoke",
            "--url",
            &config.rpc_url,
            "--contract-address",
            contract_address,
            "--function",
            "reset",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().unwrap();
    let reader = BufReader::new(stdout);

    for line in reader.lines() {
        let line = line?;
        println!("{}", line);
    }

    let status = child.wait()?;

    let stderr = child.stderr.take().unwrap();
    let stderr_reader = BufReader::new(stderr);
    for line in stderr_reader.lines() {
        let line = line?;
        if !line.is_empty() {
            error!("{}", line);
        }
    }

    if !status.success() {
        return Err("Reset failed. Check the error above.".into());
    }

    println!("Game {} reset successfully!", game_id);

    let mut child = Command::new("sncast")
        .current_dir(CONTRACT_PATH)
        .args([
            "--wait",
            "call",
            "--url",
            &config.rpc_url,
            "--contract-address",
            contract_address,
            "--function",
            "get_next_game_id",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child.stdout.take().unwrap();
    let reader = BufReader::new(stdout);

    for line in reader.lines() {
        let line = line?;
        println!("{}", line);
    }

    Ok(())
}

// ── starknet-rs ───────────────────────────────────────────────────────────────

async fn reset_game_rs(
    config: &Config,
    game_id: &str,
    account: &SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>,
) -> Result<(), Box<dyn std::error::Error>> {
    let contract_address = Felt::from_hex(config.contract_address()?)?;

    println!("Resetting game {}...", game_id);

    let provider = account.provider();
    let result = account
        .execute_v3(vec![Call {
            to: contract_address,
            selector: get_selector_from_name("reset")?,
            calldata: vec![],
        }])
        .send()
        .await?;

    wait_for_tx(provider, result.transaction_hash).await?;
    println!("Game {} reset successfully!", game_id);

    let next_game_id = provider
        .call(
            FunctionCall {
                contract_address,
                entry_point_selector: get_selector_from_name("get_next_game_id")?,
                calldata: vec![],
            },
            BlockId::Tag(BlockTag::Latest),
        )
        .await?;

    println!("Next game id: {}", next_game_id[0]);

    Ok(())
}

// ── main ──────────────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    colog::init();

    let args: Vec<String> = env::args().collect();
    let game_id = if args.len() != 2 { "1" } else { &args[1] };

    let config = Config::from_env()?;

    if config.use_sncast {
        check_sncast()?;
        reset_game_sncast(&config, game_id)?;
    } else {
        let provider = config.provider();
        let account = config.deployer_account(provider).await?;
        reset_game_rs(&config, game_id, &account).await?;
    }

    Ok(())
}
