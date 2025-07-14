// Test Utilities for Valence Protocol Integration Tests
// ====================================================
//
// This module provides shared utilities for integration tests:
// 
// 1. LocalValidator - Manages a local solana-test-validator instance
//    - Automatically starts and stops the validator
//    - Provides airdrop functionality for test accounts
//    - Cleans up ledger directory on drop
//
// 2. DeployedPrograms - Handles program deployment for tests
//    - Deploys Registry, Shard, and Test Function programs
//    - Uses environment-specific keypairs for deterministic addresses
//    - Validates deployment success
//
// These utilities abstract away the boilerplate of setting up a test
// environment, allowing tests to focus on protocol behavior.

use std::process::{Command, Child};
use std::thread;
use std::time::Duration;
use solana_sdk::pubkey::Pubkey;
use std::fs;
use std::path::{Path, PathBuf};
use std::env;


pub struct LocalValidator {
    process: Child,
    ledger_dir: PathBuf,
}

impl LocalValidator {
    pub fn start() -> Result<Self, Box<dyn std::error::Error>> {
        // Create temporary directory for ledger
        let ledger_dir = std::env::temp_dir().join(format!("valence-test-{}", std::process::id()));
        fs::create_dir_all(&ledger_dir)?;
        
        println!("Starting local validator with ledger at: {:?}", ledger_dir);
        
        // Check if we're already in nix environment
        if env::var("IN_NIX_SHELL").is_err() {
            return Err("This test must be run inside nix develop environment. Run: nix develop -c cargo test".into());
        }
        
        // Start validator directly (we're already in nix environment)
        let mut cmd = Command::new("solana-test-validator");
        cmd
            .arg("--ledger")
            .arg(&ledger_dir)
            .arg("--rpc-port")
            .arg("8899")
            .arg("--quiet");
            
        let process = cmd.spawn()?;
        
        // Wait for validator to start
        thread::sleep(Duration::from_secs(5));
        
        // Configure CLI to use local validator
        Command::new("solana")
            .args(["config", "set", "--url", "http://localhost:8899"])
            .output()?;
            
        // Create a test keypair in a temporary location
        let test_keypair_path = std::env::temp_dir().join(format!("valence-test-keypair-{}.json", std::process::id()));
        
        let keygen_output = Command::new("solana-keygen")
            .args(["new", "--no-passphrase", "--force", "-o", test_keypair_path.to_str().unwrap()])
            .output()?;
            
        if !keygen_output.status.success() {
            return Err(format!("Failed to create keypair: {}", 
                String::from_utf8_lossy(&keygen_output.stderr)).into());
        }
        
        // Configure CLI to use this keypair
        Command::new("solana")
            .args(["config", "set", "--keypair", test_keypair_path.to_str().unwrap()])
            .output()?;
        
        // Wait a bit more and check validator is ready
        for _ in 0..10 {
            let result = Command::new("solana")
                .args(["cluster-version"])
                .output();
                
            if let Ok(output) = result {
                if output.status.success() {
                    break;
                }
            }
            thread::sleep(Duration::from_secs(1));
        }
            
        Ok(LocalValidator {
            process,
            ledger_dir,
        })
    }
    
    pub fn airdrop(&self, pubkey: &Pubkey, sol: u64) -> Result<(), Box<dyn std::error::Error>> {
        Command::new("solana")
            .args(["airdrop", &sol.to_string(), &pubkey.to_string()])
            .output()?;
        Ok(())
    }
}

impl Drop for LocalValidator {
    fn drop(&mut self) {
        // Kill validator process
        let _ = self.process.kill();
        let _ = self.process.wait();
        
        // Clean up ledger directory
        let _ = fs::remove_dir_all(&self.ledger_dir);
    }
}

pub struct DeployedPrograms {
    pub registry_id: Pubkey,
    pub shard_id: Pubkey,
    pub test_function_id: Pubkey,
}

impl DeployedPrograms {
    pub fn deploy() -> Result<Self, Box<dyn std::error::Error>> {
        // Get the workspace root directory (go up from tests/integration)
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .ok_or("Could not find workspace root")?;
        
        // First check if programs are built
        let registry_path = workspace_root.join("target/sbf-solana-solana/release/registry.so");
        let shard_path = workspace_root.join("target/sbf-solana-solana/release/shard.so");
        let test_function_path = workspace_root.join("target/sbf-solana-solana/release/test_function.so");
        
        if !registry_path.exists() || !shard_path.exists() || !test_function_path.exists() {
            return Err("Programs not built. Run 'nix develop -c bash scripts/build-with-keys.sh' first".into());
        }
        
        // Use the test keypairs
        let registry_keypair_path = workspace_root.join("tests/integration/keypairs/registry-keypair.json");
        let shard_keypair_path = workspace_root.join("tests/integration/keypairs/shard-keypair.json");
        let test_function_keypair_path = workspace_root.join("tests/integration/keypairs/test_function-keypair.json");
        
        if !registry_keypair_path.exists() || !shard_keypair_path.exists() || !test_function_keypair_path.exists() {
            return Err("Test keypairs not found. Make sure tests/integration/keypairs/ contains all required keypair files".into());
        }
        
        // Get the program IDs from keypairs
        let registry_id = "E3geaX2kFBSvHV4co5odHsRW737NJjySziGXk8jXJCqV".parse::<Pubkey>()?;
        let shard_id = "B2UgDMshe2sug7qTv4DseFNz6ipRSKPbqc9j98TAWJuo".parse::<Pubkey>()?;
        let test_function_id = "8r2SeUcUmdzXHuvsNDsNxCPLkn8w6Jz9z1wtLk3ChzNR".parse::<Pubkey>()?;
        
        // Airdrop to default keypair for deployment
        Command::new("solana")
            .args(["airdrop", "10"])
            .output()?;
        // Deploy registry with specific keypair
        println!("Deploying registry program to: {}", registry_id);
        let registry_output = Command::new("solana")
            .args([
                "program", 
                "deploy", 
                "--program-id",
                registry_keypair_path.to_str().unwrap(),
                registry_path.to_str().unwrap()
            ])
            .output()?;
            
        if !registry_output.status.success() {
            return Err(format!("Failed to deploy registry: {}", 
                String::from_utf8_lossy(&registry_output.stderr)).into());
        }
        
        // Deploy shard with specific keypair
        println!("Deploying shard program to: {}", shard_id);
        let shard_output = Command::new("solana")
            .args([
                "program", 
                "deploy",
                "--program-id",
                shard_keypair_path.to_str().unwrap(),
                shard_path.to_str().unwrap()
            ])
            .output()?;
            
        if !shard_output.status.success() {
            return Err(format!("Failed to deploy shard: {}", 
                String::from_utf8_lossy(&shard_output.stderr)).into());
        }
        
        // Deploy test function with specific keypair
        println!("Deploying test function program to: {}", test_function_id);
        let test_function_output = Command::new("solana")
            .args([
                "program", 
                "deploy",
                "--program-id",
                test_function_keypair_path.to_str().unwrap(),
                test_function_path.to_str().unwrap()
            ])
            .output()?;
            
        if !test_function_output.status.success() {
            return Err(format!("Failed to deploy test function: {}", 
                String::from_utf8_lossy(&test_function_output.stderr)).into());
        }
        
        Ok(DeployedPrograms {
            registry_id,
            shard_id,
            test_function_id,
        })
    }
}


