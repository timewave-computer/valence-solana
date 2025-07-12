//! Shard errors

use anchor_lang::prelude::*;

#[error_code]
pub enum ShardError {
    #[msg("Shard is paused")]
    ShardPaused,
    
    #[msg("Invalid session request")]
    InvalidSessionRequest,
    
    #[msg("Session not found")]
    SessionNotFound,
    
    #[msg("Session not active")]
    SessionNotActive,
    
    #[msg("Unauthorized access")]
    Unauthorized,
    
    #[msg("Invalid bundle")]
    InvalidBundle,
    
    #[msg("Too many operations in bundle")]
    TooManyOperations,
    
    #[msg("Function not found in registry")]
    FunctionNotFound,
    
    #[msg("Execution failed")]
    ExecutionFailed,
    
    #[msg("Invalid state hash")]
    InvalidStateHash,
    
    #[msg("Diff mismatch")]
    DiffMismatch,
    
    #[msg("Bundle already exists")]
    BundleAlreadyExists,
    
    #[msg("Bundle not found")]
    BundleNotFound,
    
    #[msg("Invalid checkpoint")]
    InvalidCheckpoint,
    
    #[msg("Capability not granted")]
    CapabilityNotGranted,
    
    #[msg("Context isolation violation")]
    ContextIsolationViolation,
    
    #[msg("Verification failed")]
    VerificationFailed,
    
    #[msg("Function mismatch")]
    FunctionMismatch,
    
    #[msg("Insufficient capabilities for function execution")]
    InsufficientCapabilities,
    
    #[msg("Session has already been consumed")]
    SessionAlreadyConsumed,
}