// Registry error types aligned with valence-kernel design

use thiserror::Error;

/// Registry error types
#[derive(Debug, Error, Clone)]
pub enum RegistryError {
    #[error("Function not found: {0}")]
    FunctionNotFound(String),
    
    #[error("Shard not found: {0}")]
    ShardNotFound(String),
    
    #[error("Invalid content hash")]
    InvalidContentHash,
    
    #[error("Function already exists")]
    FunctionAlreadyExists,
    
    #[error("Shard already exists")]
    ShardAlreadyExists,
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Storage error: {0}")]
    StorageError(String),
    
    #[error("Invalid registry ID: {0}")]
    InvalidRegistryId(u64),
    
    #[error("Registry capacity exceeded")]
    CapacityExceeded,
    
    #[error("Invalid function signature")]
    InvalidFunctionSignature,
    
    #[error("Invalid function metadata")]
    InvalidMetadata,
}

impl From<serde_json::Error> for RegistryError {
    fn from(err: serde_json::Error) -> Self {
        RegistryError::SerializationError(err.to_string())
    }
}

/// Registry result type
pub type Result<T> = std::result::Result<T, RegistryError>;