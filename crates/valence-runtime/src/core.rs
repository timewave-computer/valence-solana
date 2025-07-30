//! Core runtime types: configuration and error handling

use solana_sdk::commitment_config::CommitmentConfig;
use thiserror::Error;

// ================================
// Configuration Types
// ================================

/// Runtime configuration
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// RPC endpoint URL
    pub rpc_url: String,

    /// WebSocket endpoint URL
    pub ws_url: String,

    /// Commitment level for RPC queries
    pub commitment: CommitmentConfig,

    /// Maximum retries for RPC calls
    pub max_retries: u32,

    /// Enable transaction simulation before submission
    pub enable_simulation: bool,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            rpc_url: "https://api.mainnet-beta.solana.com".to_string(),
            ws_url: "wss://api.mainnet-beta.solana.com".to_string(),
            commitment: CommitmentConfig::confirmed(),
            max_retries: 3,
            enable_simulation: true,
        }
    }
}

// ================================
// Error Types
// ================================

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("RPC error: {0}")]
    Rpc(Box<solana_client::client_error::ClientError>),

    #[error("WebSocket error: {0}")]
    WebSocket(Box<tungstenite::Error>),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invalid account data")]
    InvalidAccountData,

    #[error("Transaction building failed: {0}")]
    TransactionBuildError(String),

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("Authentication failed")]
    AuthenticationFailed,

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Coordination error: {0}")]
    CoordinationError(String),

    #[error("Security policy violation: {0}")]
    SecurityViolation(String),

    #[error("State validation failed: {0}")]
    StateValidationFailed(String),

    #[error("Timeout occurred")]
    Timeout,
}

impl From<solana_client::client_error::ClientError> for RuntimeError {
    fn from(err: solana_client::client_error::ClientError) -> Self {
        Self::Rpc(Box::new(err))
    }
}

impl From<tungstenite::Error> for RuntimeError {
    fn from(err: tokio_tungstenite::tungstenite::Error) -> Self {
        Self::WebSocket(Box::new(err))
    }
}

impl From<std::io::Error> for RuntimeError {
    fn from(err: std::io::Error) -> Self {
        Self::TransactionBuildError(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, RuntimeError>;