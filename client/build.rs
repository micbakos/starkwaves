use std::collections::HashMap;
use cainome::rs::{Abigen, ExecutionVersion};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const BUILD_RELEASE: bool = false;

fn main() {
    let scarb_check = Command::new("scarb").arg("--version").output();

    if scarb_check.is_err() {
        println!("cargo:warning=Scarb not found. Please install Scarb to compile Cairo code.");
        println!("cargo:warning=Visit: https://docs.swmansion.com/scarb/download");
        println!("cargo:warning=Skipping Cairo compilation...");
        panic!("Failed to compile Cairo compilation");
    }

    #[cfg(feature = "merkle-build")]
    {
        println!("cargo:rerun-if-changed=../contract/merkle/Scarb.toml");
        println!("cargo:rerun-if-changed=../contract/merkle/src");
        println!("cargo:rerun-if-changed=../contract/merkle/target");

        let sierra_file = build_merkle(BUILD_RELEASE);
        println!(
            "cargo:rustc-env=MERKLE_SIERRA_PATH={}",
            sierra_file.display()
        );
    }

    println!("cargo:rerun-if-changed=../contract/src");
    println!("cargo:rerun-if-changed=../contract/target");
    println!("cargo:rerun-if-changed=../contract/Scarb.toml");


    // let contract_dir = Path::new("../contract");
    // let mut scarb_args: Vec<&str> = vec![];
    // scarb_args.push("build");
    // let output = Command::new("scarb")
    //     .current_dir(contract_dir)
    //     .args(scarb_args)
    //     .output()
    //     .expect("Failed to execute scarb build");
    //
    // if !output.status.success() {
    //     let stderr = String::from_utf8_lossy(&output.stderr);
    //     panic!("Cairo compilation failed:\n{}", stderr);
    // }
    //
    // let target_dir = if BUILD_RELEASE {
    //     contract_dir.join("target/release")
    // } else {
    //     contract_dir.join("target/dev")
    // };
    //
    // let contract = find_json_file(&target_dir, "starkwaves_Starkwaves.contract_class.json")
    //     .expect("Could not find Contract output file. Check Scarb build output.");
    //
    // let output_dir = PathBuf::from("src/types/contract");
    //
    // let output_path = output_dir.join("starkwaves.rs");
    // let abigen = Abigen::new("Starkwaves", contract.to_str().unwrap())
    //     .with_execution_version(ExecutionVersion::V3)
    //     .with_derives(vec![
    //         "Debug".into(),
    //         "Clone".into(),
    //     ])
    //     .with_types_aliases(HashMap::from([
    //         ("openzeppelin_access::ownable::ownable::OwnableComponent::Event".to_string(), "OwnableComponentEvent".to_string())
    //     ]));
    //
    // abigen
    //     .generate()
    //     .expect("Unable to generate Abigen.")
    //     .write_to_file(output_path.to_str().unwrap())
    //     .expect("Couldn't write to src/abi.rs");
    //
    // let generated = fs::read_to_string(&output_path).expect("Failed to read generated file");
    // let patched = generated.replace("starknet::", "starknet_rust::");
    // fs::write(&output_path, patched).expect("Failed to patch generated file");
    //
    // let mod_rs_path = output_dir.join("mod.rs");
    // if mod_rs_path.exists() {
    //     let contents = fs::read_to_string(&mod_rs_path).expect("Failed to read mod.rs");
    //     if !contents.contains("pub mod starkwaves") {
    //         let new_contents = format!("{}\npub mod starkwaves;\n", contents.trim_end());
    //         fs::write(&mod_rs_path, new_contents).expect("Failed to write to mod.rs");
    //     }
    // }
}

#[cfg(feature = "merkle-build")]
fn build_merkle(release: bool) -> PathBuf {
    let merkle_dir = Path::new("../contract/merkle");

    let mut scarb_args: Vec<&str> = vec![];
    if release {
        scarb_args.push("--release");
    }
    scarb_args.push("build");

    // Run scarb
    let output = Command::new("scarb")
        .current_dir(merkle_dir)
        .args(scarb_args)
        .output()
        .expect("Failed to execute scarb build");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("Cairo compilation failed:\n{}", stderr);
    }

    let target_dir = if release {
        merkle_dir.join("target/release")
    } else {
        merkle_dir.join("target/dev")
    };

    // Find sierra program file
    find_json_file(&target_dir, "merkle.sierra.json")
        .expect("Could not find Sierra output file. Check Scarb build output.")
}

fn find_json_file(dir: &Path, file_name: &str) -> Option<PathBuf> {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                    if name == file_name {
                        return Some(path);
                    }
                }
            }
        }
    }

    None
}
