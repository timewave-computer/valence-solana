//! Session builder service binary

use anyhow::Result;
use session_builder::{Config, run};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<()> {
    // Parse command line arguments (simplified)
    let args: Vec<String> = std::env::args().collect();
    
    let config = if args.len() > 1 {
        Config {
            rpc_url: args.get(1).cloned().unwrap_or_default(),
            keypair_path: args.get(2).cloned().unwrap_or_default(),
            shard_program_id: args.get(3)
                .and_then(|s| Pubkey::from_str(s).ok())
                .unwrap_or_default(),
            poll_interval_secs: args.get(4)
                .and_then(|s| s.parse().ok())
                .unwrap_or(5),
            max_retries: 3,
        }
    } else {
        Config::default()
    };
    
    println!("Session Builder Service Configuration:");
    println!("  RPC URL: {}", config.rpc_url);
    println!("  Keypair: {}", config.keypair_path);
    println!("  Shard Program: {}", config.shard_program_id);
    println!("  Poll Interval: {}s", config.poll_interval_secs);
    
    // Run the service
    run(config).await
}