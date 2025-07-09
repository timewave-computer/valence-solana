/// Error types for the Valence Protocol SDK
/// 
/// This module provides comprehensive error handling for all SDK operations.

use thiserror::Error;
use anchor_lang::prelude::*;

/// Main error type for the Valence SDK
#[derive(Error, Debug)]
pub enum ValenceError {
    // Context Errors (1000-1099)
    #[error("Invalid target session: {0}")]
    InvalidTargetSession(String),
    
    #[error("Invalid settlement data: {0}")]
    InvalidSettlementData(String),
    
    #[error("Missing capability ID")]
    MissingCapabilityId,
    
    // Authentication and Authorization Errors (2000-2999)
    #[error("Unauthorized access")]
    Unauthorized,
    
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    #[error("Insufficient permissions for operation")]
    InsufficientPermissions,
    
    // Validation Errors (3000-3099)
    #[error("Invalid input parameters: {0}")]
    InvalidInputParameters(String),
    
    #[error("Data validation failed: {0}")]
    DataValidationFailed(String),
    
    #[error("Constraint check failed: {0}")]
    ConstraintCheckFailed(String),
    
    #[error("Invalid account state: {0}")]
    InvalidAccountState(String),
    
    // Capability & Session Errors (4000-4099)
    #[error("Capability not found: {0}")]
    CapabilityNotFound(String),
    
    #[error("Capability not active: {0}")]
    CapabilityNotActive(String),
    
    #[error("Session not found: {0}")]
    SessionNotFound(String),
    
    #[error("Session not active: {0}")]
    SessionNotActive(String),
    
    // System & Resource Errors (6000-6099)
    #[error("Program is paused")]
    ProgramPaused,
    
    #[error("Insufficient compute units")]
    InsufficientComputeUnits,
    
    // SDK Errors (8000-8099)
    #[error("Invalid SDK configuration: {0}")]
    InvalidConfiguration(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
    
    #[error("Account not found: {0}")]
    AccountNotFound(String),
    
    #[error("Insufficient balance: {0}")]
    InsufficientBalance(String),
    
    #[error("Not implemented: {0}")]
    NotImplemented(String),
    
    // Wrapped errors
    #[error("Anchor error: {0}")]
    AnchorError(#[from] anchor_lang::error::Error),
    
    #[error("Solana client error: {0}")]
    SolanaClientError(#[from] solana_client::client_error::ClientError),
    
    #[error("Solana program error: {0}")]
    SolanaProgramError(#[from] solana_sdk::program_error::ProgramError),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] bincode::Error),
    
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Other error: {0}")]
    Other(String),
}

impl ValenceError {
    /// Get the error code for this error
    pub fn code(&self) -> u32 {
        match self {
            // Context Errors
            ValenceError::InvalidTargetSession(_) => 1000,
            ValenceError::InvalidSettlementData(_) => 1001,
            ValenceError::MissingCapabilityId => 1002,
            
            // Authentication and Authorization Errors
            ValenceError::Unauthorized => 2000,
            ValenceError::AuthenticationFailed(_) => 2001,
            ValenceError::InsufficientPermissions => 2002,
            
            // Validation Errors
            ValenceError::InvalidInputParameters(_) => 3000,
            ValenceError::DataValidationFailed(_) => 3001,
            ValenceError::ConstraintCheckFailed(_) => 3002,
            ValenceError::InvalidAccountState(_) => 3003,
            
            // Capability & Session Errors
            ValenceError::CapabilityNotFound(_) => 4000,
            ValenceError::CapabilityNotActive(_) => 4001,
            ValenceError::SessionNotFound(_) => 4003,
            ValenceError::SessionNotActive(_) => 4004,
            
            // System & Resource Errors
            ValenceError::ProgramPaused => 6000,
            ValenceError::InsufficientComputeUnits => 6001,
            
            // SDK Errors
            ValenceError::InvalidConfiguration(_) => 8000,
            ValenceError::NetworkError(_) => 8001,
            ValenceError::TransactionFailed(_) => 8002,
            ValenceError::AccountNotFound(_) => 8003,
            ValenceError::InsufficientBalance(_) => 8004,
            ValenceError::NotImplemented(_) => 8005,
            
            // Wrapped errors use generic codes
            ValenceError::AnchorError(_) => 9000,
            ValenceError::SolanaClientError(_) => 9001,
            ValenceError::SolanaProgramError(_) => 9002,
            ValenceError::SerializationError(_) => 9003,
            ValenceError::JsonError(_) => 9004,
            ValenceError::IoError(_) => 9005,
            ValenceError::Other(_) => 9999,
        }
    }
    
    /// Create a new network error
    pub fn network_error<T: std::fmt::Display>(msg: T) -> Self {
        ValenceError::NetworkError(msg.to_string())
    }
    
    /// Create a new transaction failed error
    pub fn transaction_failed<T: std::fmt::Display>(msg: T) -> Self {
        ValenceError::TransactionFailed(msg.to_string())
    }
    
    /// Create a new configuration error
    pub fn invalid_configuration<T: std::fmt::Display>(msg: T) -> Self {
        ValenceError::InvalidConfiguration(msg.to_string())
    }
    
    /// Create a new validation error
    pub fn validation_failed<T: std::fmt::Display>(msg: T) -> Self {
        ValenceError::DataValidationFailed(msg.to_string())
    }
}

/// Result type for SDK operations
pub type ValenceResult<T> = std::result::Result<T, ValenceError>;

/// Helper macro for creating validation errors
#[macro_export]
macro_rules! validate {
    ($condition:expr, $msg:expr) => {
        if !$condition {
            return Err(ValenceError::DataValidationFailed($msg.to_string()));
        }
    };
}

/// Helper macro for creating configuration errors
#[macro_export]
macro_rules! config_error {
    ($msg:expr) => {
        ValenceError::InvalidConfiguration($msg.to_string())
    };
}

/// Helper macro for creating network errors
#[macro_export]
macro_rules! network_error {
    ($msg:expr) => {
        ValenceError::NetworkError($msg.to_string())
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        // Test that error codes are consistent
        assert_eq!(ValenceError::Unauthorized.code(), 2000);
        assert_eq!(ValenceError::CapabilityNotFound("test".to_string()).code(), 4000);
        assert_eq!(ValenceError::ProgramPaused.code(), 6000);
        assert_eq!(ValenceError::InvalidConfiguration("test".to_string()).code(), 8000);
    }
    
    #[test]
    fn test_error_creation() {
        // Test helper methods
        let network_err = ValenceError::network_error("Connection failed");
        assert_eq!(network_err.code(), 8001);
        
        let config_err = ValenceError::invalid_configuration("Invalid program ID");
        assert_eq!(config_err.code(), 8000);
        
        let validation_err = ValenceError::validation_failed("Invalid input");
        assert_eq!(validation_err.code(), 3001);
    }
} 