use anchor_lang::prelude::*;

#[error_code]
pub enum AuthorizationError {
    #[msg("Not authorized to perform this action")]
    NotAuthorized,
    
    #[msg("Invalid parameters provided")]
    InvalidParameters,
    
    #[msg("Authorization is not active")]
    AuthorizationInactive,
    
    #[msg("Authorization has expired")]
    AuthorizationExpired,
    
    #[msg("Authorization not yet valid")]
    AuthorizationNotYetValid,
    
    #[msg("Maximum concurrent executions reached")]
    MaxConcurrentExecutionsReached,
    
    #[msg("Invalid authorization label")]
    InvalidAuthorizationLabel,
    
    #[msg("Unauthorized sender")]
    UnauthorizedSender,
    
    #[msg("Invalid timestamp")]
    InvalidTimestamp,
    
    #[msg("Invalid processor program ID")]
    InvalidProcessorProgramId,
    
    #[msg("Invalid registry program ID")]
    InvalidRegistryProgramId,
    
    #[msg("Empty message batch")]
    EmptyMessageBatch,
    
    #[msg("Message too large")]
    MessageTooLarge,
    
    #[msg("Callback from unauthorized program")]
    UnauthorizedCallback,
    
    #[msg("Execution not found")]
    ExecutionNotFound,
} 