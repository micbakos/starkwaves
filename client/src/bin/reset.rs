#[path = "common.rs"]
mod common;

use std::env;
use std::io::{BufRead, BufReader};
use std::process::{exit, Command, Stdio};
use log::{error, info};
use common::{check_sncast, Config, CONTRACT_PATH};

fn reset_game(config: &Config, game_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let contract_address = config.contract_address()?;

    println!("Resetting game {}...", game_id);

    let mut child = Command::new("sncast")
        .current_dir(CONTRACT_PATH)
        .args([
            "--account", &config.account_name,
            "--wait",
            "invoke",
            "--url", &config.rpc_url,
            "--contract-address", contract_address,
            "--function", "reset",
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
            "--url", &config.rpc_url,
            "--contract-address", contract_address,
            "--function", "get_next_game_id",
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    colog::init();

    let args: Vec<String> = env::args().collect();

    let game_id = if args.len() != 2 {
        "1"
    } else {
        &args[1]
    };

    check_sncast()?;
    let config = Config::from_env()?;
    reset_game(&config, game_id)?;

    Ok(())
}
