use thiserror::Error;

// ================================
// Registry Error Types
// ================================

#[derive(Error, Debug)]
pub enum RegistryError {
    #[error("Function not found: {0}")]
    FunctionNotFound(String),

    #[error("Protocol not found: {0}")]
    ProtocolNotFound(String),

    #[error("Invalid content hash")]
    InvalidContentHash,

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("IPFS error: {0}")]
    IpfsError(String),

    #[error("Function already exists: {0}")]
    FunctionAlreadyExists(String),

    #[error("Invalid function metadata")]
    InvalidMetadata,
}

pub type Result<T> = std::result::Result<T, RegistryError>;
