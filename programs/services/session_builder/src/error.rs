//! Error types for the Session Builder service

use solana_sdk::pubkey::Pubkey;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SessionBuilderError {
    #[error("Service is already running")]
    AlreadyRunning,
    
    #[error("Service is not running")]
    NotRunning,
    
    #[error("Account already exists: {0}")]
    AccountAlreadyExists(Pubkey),
    
    #[error("Account verification failed: {0}")]
    AccountVerificationFailed(String),
    
    #[error("RPC connection error: {0}")]
    RpcError(String),
    
    #[error("Keypair error: {0}")]
    KeypairError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Event monitoring error: {0}")]
    EventMonitorError(String),
    
    #[error("Transaction error: {0}")]
    TransactionError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Timeout error: {0}")]
    TimeoutError(String),
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Insufficient funds for account creation")]
    InsufficientFunds,
    
    #[error("Invalid program ID: {0}")]
    InvalidProgramId(String),
    
    #[error("Event parsing error: {0}")]
    EventParsingError(String),
} 