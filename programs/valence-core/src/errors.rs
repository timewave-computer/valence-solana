use anchor_lang::prelude::*;

/// Core error types for valence-core
#[error_code]
pub enum CoreError {
    // --- Input Validation Errors
    
    #[msg("Parameters too large")]
    ParamsTooLarge,
    
    #[msg("Metadata too large")]
    MetadataTooLarge,
    
    #[msg("Data too large")]
    DataTooLarge,
    
    #[msg("Code too large")]
    CodeTooLarge,
    
    #[msg("Invalid operation data")]
    InvalidOperationData,
    
    #[msg("Invalid condition type")]
    InvalidCondition,
    
    #[msg("Invalid metadata format")]
    InvalidMetadata,
    
    #[msg("Invalid cache format")]
    InvalidCacheFormat,
    
    // --- Session Management Errors
    
    #[msg("Session already consumed")]
    SessionConsumed,
    
    #[msg("Not session owner")]
    NotOwner,
    
    #[msg("Too many accounts in session")]
    TooManyAccounts,
    
    #[msg("Session not consumed - cannot close")]
    SessionNotConsumed,
    
    #[msg("Invalid parent session")]
    InvalidParentSession,
    
    #[msg("Parent session not provided")]
    ParentSessionNotProvided,
    
    #[msg("Parent session access denied")]
    ParentSessionAccessDenied,
    
    // --- Account Lifecycle Errors
    
    #[msg("Account expired")]
    AccountExpired,
    
    #[msg("Usage count overflow")]
    UsageOverflow,
    
    #[msg("Usage limit exceeded")]
    UsageLimitExceeded,
    
    #[msg("Cannot close account - not expired and not authorized")]
    CannotClose,
    
    #[msg("Account key mismatch")]
    AccountMismatch,
    
    #[msg("Insufficient accounts provided")]
    InsufficientAccounts,
    
    // --- Security and Authorization Errors
    
    #[msg("Verification failed")]
    VerificationFailed,
    
    #[msg("Invalid nonce")]
    InvalidNonce,
    
    #[msg("State nonce overflow")]
    NonceOverflow,
    
    #[msg("Missing required signer")]
    MissingRequiredSigner,
    
    #[msg("Unauthorized")]
    Unauthorized,
    
    // --- Execution and System Errors
    
    #[msg("Insufficient CPI call depth remaining")]
    InsufficientCallDepth,
    
    #[msg("Execution failed")]
    ExecutionFailed,
    
    #[msg("Code hash mismatch")]
    CodeHashMismatch,
}