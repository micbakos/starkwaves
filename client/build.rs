use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    // Rerun if Cairo source changes
    // println!("cargo:rerun-if-changed=../validator/src/lib.cairo");
    // println!("cargo:rerun-if-changed=../validator/src/types.cairo");
    // println!("cargo:rerun-if-changed=../validator/Scarb.toml");


    println!("cargo:warning=Building cairo");
    let sierra_file = build_validator();

    // Tell Cargo where the files are for runtime loading
    println!("cargo:rustc-env=VALIDATOR_SIERRA_PATH={}", sierra_file.display());
}

fn build_validator() -> PathBuf {
    let validator_dir = Path::new("../validator");

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

    // Run scarb
    let output = Command::new("scarb")
        .current_dir(validator_dir)
        .arg("build")
        .output()
        .expect("Failed to execute scarb build");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("Cairo compilation failed:\n{}", stderr);
    }

    // Find sierra program file
    let target_dir = validator_dir.join("target/dev");
    find_sierra_file(&target_dir)
        .expect("Could not find Sierra output file. Check Scarb build output.")

    // Copy Sierra file to a known location for embedding
    // // Step 5: Extract Sierra program from Contract Class JSON
    // println!("Extracting Sierra program from contract class...");
    //
    // let contract_class_json = fs::read_to_string(&dest_path)
    //     .expect("Failed to read contract class JSON");
    //
    // // Parse the contract class using cairo-lang-starknet-classes
    // let contract_class: ContractClass = serde_json::from_str(&contract_class_json)
    //     .expect("Failed to parse contract class JSON");
    //
    // // Extract the Sierra program from the contract class
    // let sierra_program = contract_class.extract_sierra_program()
    //     .expect("Failed to extract Sierra program from contract class");
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
