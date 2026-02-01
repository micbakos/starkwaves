use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    #[cfg(feature = "merkle-build")]
    {
        println!("cargo:rerun-if-changed=../contract/merkle/Scarb.toml");
        println!("cargo:rerun-if-changed=../contract/merkle/src");
        println!("cargo:rerun-if-changed=../contract/merkle/target");

        let sierra_file = build_merkle(false);
        println!("cargo:rustc-env=MERKLE_SIERRA_PATH={}", sierra_file.display());
    }
}

#[cfg(feature = "merkle-build")]
fn build_merkle(release: bool) -> PathBuf {
    let merkle_dir = Path::new("../contract/merkle");

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
    find_sierra_file(&target_dir)
        .expect("Could not find Sierra output file. Check Scarb build output.")
}

#[cfg(feature = "merkle-build")]
fn find_sierra_file(dir: &Path) -> Option<PathBuf> {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
                    // Look for the .sierra.json output (library, not contract)
                    if name.ends_with(".sierra.json")
                        && !name.contains("contract_class")
                        && !name.contains("tests")
                        && !name.contains("unittest") {
                        return Some(path)
                    }
                }
            }
        }
    }

    None
}
