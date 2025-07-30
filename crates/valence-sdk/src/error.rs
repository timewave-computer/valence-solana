use anchor_client::ClientError;
use anchor_lang::prelude::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SdkError {
    #[error("Anchor client error: {0}")]
    AnchorClient(Box<ClientError>),

    #[error("Solana client error: {0}")]
    SolanaClient(String),

    #[error("Program error: {0}")]
    Program(#[from] ProgramError),

    #[error("Invalid session configuration")]
    InvalidSessionConfig,


    #[error("Transaction failed: {0}")]
    TransactionFailed(String),

    #[error("Account not found: {0}")]
    AccountNotFound(String),
    
    #[error("Account already registered")]
    AccountAlreadyRegistered,
    
    #[error("Account already borrowed")]
    AccountAlreadyBorrowed,
    
    #[error("Account not borrowed")]
    AccountNotBorrowed,
    
    #[error("Invalid account index")]
    InvalidAccountIndex,

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Compute budget exceeded: estimated {estimated}, limit {limit}")]
    ComputeBudgetExceeded { estimated: u64, limit: u64 },
    
    #[error("Unauthorized")]
    Unauthorized,
    
    #[error("Session inactive")]
    SessionInactive,
    
    #[error("Arithmetic overflow")]
    Overflow,
    
    #[error("Stale reference: ownership has changed")]
    StaleReference,
    
    #[error("Capability expired")]
    CapabilityExpired,
    
    #[error("Capability exhausted: no uses remaining")]
    CapabilityExhausted,
}

impl From<ClientError> for SdkError {
    fn from(err: ClientError) -> Self {
        Self::AnchorClient(Box::new(err))
    }
}