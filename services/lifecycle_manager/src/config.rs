//! Service configuration

use anyhow::{Result, Context};
use solana_sdk::signature::Keypair;
use std::sync::Arc;

#[derive(Clone)]
pub struct Config {
    /// Solana RPC URL
    pub rpc_url: String,
    
    /// Solana WebSocket URL
    pub ws_url: String,
    
    /// Service wallet keypair
    pub wallet_keypair: Arc<Keypair>,
    
    /// Database connection URL
    pub database_url: String,
    
    /// Message queue URL (RabbitMQ)
    pub amqp_url: String,
    
    /// Shard program ID
    pub shard_program_id: solana_sdk::pubkey::Pubkey,
    
    /// API server port
    pub api_port: u16,
    
    /// Account request poll interval (seconds)
    pub poll_interval: u64,
    
    /// Maximum accounts per session
    pub max_accounts_per_session: usize,
    
    /// Session consumption timeout (seconds)
    pub consumption_timeout: u64,
    
    /// Enable automatic session progression
    pub auto_progress: bool,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        // Load wallet keypair
        let wallet_path = std::env::var("WALLET_PATH")
            .unwrap_or_else(|_| "~/.config/solana/id.json".to_string());
        let wallet_path = shellexpand::tilde(&wallet_path).to_string();
        let wallet_keypair = Arc::new(
            Keypair::from_bytes(&std::fs::read(&wallet_path)?)
                .context("Failed to load wallet keypair")?
        );
        
        // Parse program ID
        let shard_program_id = std::env::var("SHARD_PROGRAM_ID")
            .context("SHARD_PROGRAM_ID not set")?
            .parse()
            .context("Invalid SHARD_PROGRAM_ID")?;
        
        Ok(Config {
            rpc_url: std::env::var("RPC_URL")
                .unwrap_or_else(|_| "http://localhost:8899".to_string()),
            
            ws_url: std::env::var("WS_URL")
                .unwrap_or_else(|_| "ws://localhost:8900".to_string()),
            
            wallet_keypair,
            
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://localhost/valence_lifecycle".to_string()),
            
            amqp_url: std::env::var("AMQP_URL")
                .unwrap_or_else(|_| "amqp://localhost:5672".to_string()),
            
            shard_program_id,
            
            api_port: std::env::var("API_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .context("Invalid API_PORT")?,
            
            poll_interval: std::env::var("POLL_INTERVAL")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .context("Invalid POLL_INTERVAL")?,
            
            max_accounts_per_session: std::env::var("MAX_ACCOUNTS_PER_SESSION")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .context("Invalid MAX_ACCOUNTS_PER_SESSION")?,
            
            consumption_timeout: std::env::var("CONSUMPTION_TIMEOUT")
                .unwrap_or_else(|_| "300".to_string())
                .parse()
                .context("Invalid CONSUMPTION_TIMEOUT")?,
            
            auto_progress: std::env::var("AUTO_PROGRESS")
                .unwrap_or_else(|_| "false".to_string())
                .parse()
                .context("Invalid AUTO_PROGRESS")?,
        })
    }
}