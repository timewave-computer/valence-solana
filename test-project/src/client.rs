use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Echo Program Client");
    println!("==================");
    
    let rpc_url = "http://localhost:8899";
    let client = RpcClient::new_with_commitment(rpc_url, CommitmentConfig::confirmed());
    
    // Load keypair
    let keypair_path = std::env::var("HOME")? + "/.config/solana/id.json";
    let keypair_bytes: Vec<u8> = serde_json::from_str(&std::fs::read_to_string(&keypair_path)?)?;
    let payer = Keypair::from_bytes(&keypair_bytes)?;
    
    println!("Using RPC: {}", rpc_url);
    println!("Payer: {}", payer.pubkey());
    
    // Get program ID from environment
    let program_id = std::env::var("SHARD_PROGRAM_ID")
        .or_else(|_| {
            std::fs::read_to_string(".valence-env")
                .ok()
                .and_then(|content| {
                    content.lines()
                        .find(|line| line.starts_with("export SHARD_PROGRAM_ID="))
                        .and_then(|line| line.split('=').nth(1))
                        .map(|s| s.to_string())
                })
                .ok_or_else(|| std::env::VarError::NotPresent)
        })
        .map(|s| Pubkey::from_str(&s).expect("Invalid program ID"))
        .expect("SHARD_PROGRAM_ID not found");
    
    println!("Echo Program: {}", program_id);
    
    // First, initialize the program
    println!("\nInitializing program...");
    let init_data = vec![0u8]; // Instruction 0 = initialize
    
    let init_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(payer.pubkey(), true),
        ],
        data: init_data,
    };
    
    let recent_blockhash = client.get_latest_blockhash()?;
    let init_tx = Transaction::new_signed_with_payer(
        &[init_ix],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );
    
    match client.send_and_confirm_transaction(&init_tx) {
        Ok(signature) => {
            println!("Initialized! Signature: {}", signature);
        }
        Err(e) => {
            println!("Init error (may already be initialized): {}", e);
        }
    }
    
    // Now send echo message
    let message = "Hello, Solana!";
    println!("\nSending echo message: {}", message);
    
    let mut echo_data = vec![1u8]; // Instruction 1 = echo
    echo_data.extend_from_slice(message.as_bytes());
    
    let echo_ix = Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new_readonly(payer.pubkey(), true),
        ],
        data: echo_data,
    };
    
    let recent_blockhash = client.get_latest_blockhash()?;
    let echo_tx = Transaction::new_signed_with_payer(
        &[echo_ix],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );
    
    match client.send_and_confirm_transaction(&echo_tx) {
        Ok(signature) => {
            println!("\nSuccess! Transaction signature: {}", signature);
        }
        Err(e) => {
            println!("\nError: {}", e);
            return Err(e.into());
        }
    }
    
    println!("\nClient execution complete!");
    Ok(())
}
