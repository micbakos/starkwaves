use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=../contract/target/dev");
    println!("cargo:rerun-if-changed=../contract/target/release");
    println!("cargo:rerun-if-changed=../contract/src");
    println!("cargo:rerun-if-changed=../contract/Scarb.toml");

    #[cfg(feature = "cairo-build")]
    {
        let sierra_file = build_contract(false);
        println!("cargo:rustc-env=CONTRACT_SIERRA_PATH={}", sierra_file.display());
    }
}

fn build_contract(release: bool) -> PathBuf {
    let validator_dir = Path::new("../contract");

    // Check if scarb is available
    let scarb_check = Command::new("scarb")
        .arg("--version")
        .output();

    if scarb_check.is_err() {
        println!("cargo:warning=Scarb not found. Please install Scarb to compile Cairo code.");
        println!("cargo:warning=Visit: https://docs.swmansion.com/scarb/download");
        println!("cargo:warning=Skipping Cairo compilation...");
        panic!("Failed to compile Cairo compilation");
    }

    let mut scarb_args: Vec<&str> = vec![];
    if release {
        scarb_args.push("--release");
    }
    scarb_args.push("build");

    // Run scarb
    let output = Command::new("scarb")
        .current_dir(validator_dir)
        .args(scarb_args)
        .output()
        .expect("Failed to execute scarb build");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("Cairo compilation failed:\n{}", stderr);
    }

    let target_dir = if release {
        validator_dir.join("target/release")
    } else {
        validator_dir.join("target/dev")
    };

    // Find sierra program file
    find_sierra_file(&target_dir)
        .expect("Could not find Sierra output file. Check Scarb build output.")
}

fn find_sierra_file(dir: &Path) -> Option<PathBuf> {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                    // Look for the .sierra.json output (library, not contract)
                    if name.ends_with(".sierra.json")
                        && !name.contains("contract_class")
                        && !name.contains("test")
                        && !name.contains("unittest") {
                        return Some(path)
                    }
                }
            }
        }
    }

    None
}
