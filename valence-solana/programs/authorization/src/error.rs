use anchor_lang::prelude::*;

#[error_code]
pub enum AuthorizationError {
    #[msg("You are not authorized to perform this operation")]
    NotAuthorized,
    
    #[msg("Authorization is disabled")]
    AuthorizationDisabled,
    
    #[msg("Authorization is not yet valid")]
    AuthorizationNotYetValid,
    
    #[msg("Authorization has expired")]
    AuthorizationExpired,
    
    #[msg("Too many concurrent executions for this authorization")]
    TooManyExecutions,
    
    #[msg("Invalid processor program ID")]
    InvalidProcessorProgram,
    
    #[msg("Invalid execution state")]
    InvalidExecutionState,
    
    #[msg("Label too long")]
    LabelTooLong,
    
    #[msg("Too many allowed users")]
    TooManyAllowedUsers,
    
    #[msg("Execution not found")]
    ExecutionNotFound,
    
    #[msg("Authorization not found")]
    AuthorizationNotFound,
    
    #[msg("Invalid registry program ID")]
    InvalidRegistryProgram,
    
    #[msg("Callback result mismatch")]
    CallbackResultMismatch,
    
    #[msg("Invalid message format")]
    InvalidMessageFormat,
    
    #[msg("Too many messages")]
    TooManyMessages,
} 