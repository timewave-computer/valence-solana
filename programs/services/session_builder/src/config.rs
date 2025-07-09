//! Configuration module for the Session Builder service

use anyhow::{Context, Result};
use serde::Deserialize;
use solana_sdk::{pubkey::Pubkey, signature::Keypair};
use std::{fs, path::Path, str::FromStr};

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// Solana RPC URL
    pub rpc_url: String,
    
    /// Path to the keypair file for account creation
    pub keypair_path: String,
    
    /// Session Factory program ID  
    pub session_factory_program_id: String,
    
    /// Maximum number of concurrent account creations
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent_creations: usize,
    
    /// Retry configuration
    pub retry: RetryConfig,
    
    /// Monitoring configuration
    pub monitoring: MonitoringConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retries for account creation
    #[serde(default = "default_max_retries")]
    pub max_retries: usize,
    
    /// Initial retry delay in milliseconds
    #[serde(default = "default_initial_delay_ms")]
    pub initial_delay_ms: u64,
    
    /// Maximum retry delay in milliseconds
    #[serde(default = "default_max_delay_ms")]
    pub max_delay_ms: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MonitoringConfig {
    /// Health check interval in seconds
    #[serde(default = "default_health_check_interval")]
    pub health_check_interval_secs: u64,
    
    /// Metrics collection interval in seconds
    #[serde(default = "default_metrics_interval")]
    pub metrics_interval_secs: u64,
}

impl Config {
    /// Load configuration from file or environment variables
    pub fn load(config_path: Option<&str>) -> Result<Self> {
        // Load from .env file if it exists
        if let Err(e) = dotenvy::dotenv() {
            tracing::warn!("Could not load .env file: {}", e);
        }
        
        let config = if let Some(path) = config_path {
            // Load from specified config file
            let content = fs::read_to_string(path)
                .with_context(|| format!("Failed to read config file: {}", path))?;
            
            toml::from_str(&content)
                .with_context(|| format!("Failed to parse config file: {}", path))?
        } else {
            // Load from environment variables
            Self::from_env()?
        };
        
        config.validate()?;
        Ok(config)
    }
    
    /// Load configuration from environment variables
    fn from_env() -> Result<Self> {
        let config = Config {
            rpc_url: std::env::var("RPC_URL")
                .context("RPC_URL environment variable is required")?,
            keypair_path: std::env::var("KEYPAIR_PATH")
                .context("KEYPAIR_PATH environment variable is required")?,
            session_factory_program_id: std::env::var("SESSION_FACTORY_PROGRAM_ID")
                .context("SESSION_FACTORY_PROGRAM_ID environment variable is required")?,
            max_concurrent_creations: std::env::var("MAX_CONCURRENT_CREATIONS")
                .unwrap_or_else(|_| default_max_concurrent().to_string())
                .parse()?,
            retry: RetryConfig {
                max_retries: std::env::var("MAX_RETRIES")
                    .unwrap_or_else(|_| default_max_retries().to_string())
                    .parse()?,
                initial_delay_ms: std::env::var("INITIAL_DELAY_MS")
                    .unwrap_or_else(|_| default_initial_delay_ms().to_string())
                    .parse()?,
                max_delay_ms: std::env::var("MAX_DELAY_MS")
                    .unwrap_or_else(|_| default_max_delay_ms().to_string())
                    .parse()?,
            },
            monitoring: MonitoringConfig {
                health_check_interval_secs: std::env::var("HEALTH_CHECK_INTERVAL_SECS")
                    .unwrap_or_else(|_| default_health_check_interval().to_string())
                    .parse()?,
                metrics_interval_secs: std::env::var("METRICS_INTERVAL_SECS")
                    .unwrap_or_else(|_| default_metrics_interval().to_string())
                    .parse()?,
            },
        };
        
        Ok(config)
    }
    
    /// Validate the configuration
    fn validate(&self) -> Result<()> {
        // Validate RPC URL
        if !self.rpc_url.starts_with("http") {
            anyhow::bail!("RPC URL must start with http or https");
        }
        
        // Validate keypair path exists
        if !Path::new(&self.keypair_path).exists() {
            anyhow::bail!("Keypair file does not exist: {}", self.keypair_path);
        }
        
        // Validate program IDs are valid Pubkeys
        Pubkey::from_str(&self.session_factory_program_id)
            .context("Invalid session factory program ID")?;
        
        Ok(())
    }
    
    /// Get the session factory program ID as a Pubkey
    pub fn session_factory_program_id(&self) -> Result<Pubkey> {
        Pubkey::from_str(&self.session_factory_program_id)
            .context("Invalid session factory program ID")
    }
    
    /// Load the keypair from the configured path
    pub fn load_keypair(&self) -> Result<Keypair> {
        let keypair_data = fs::read(&self.keypair_path)
            .with_context(|| format!("Failed to read keypair file: {}", self.keypair_path))?;
        
        let keypair = if keypair_data.len() == 64 {
            // Raw secret key bytes
            Keypair::from_bytes(&keypair_data)?
        } else {
            // JSON format
            let json: Vec<u8> = serde_json::from_slice(&keypair_data)
                .context("Failed to parse keypair JSON")?;
            Keypair::from_bytes(&json)?
        };
        
        Ok(keypair)
    }
}

// Default values
fn default_max_concurrent() -> usize { 10 }
fn default_max_retries() -> usize { 3 }
fn default_initial_delay_ms() -> u64 { 1000 }
fn default_max_delay_ms() -> u64 { 30000 }
fn default_health_check_interval() -> u64 { 30 }
fn default_metrics_interval() -> u64 { 60 } 