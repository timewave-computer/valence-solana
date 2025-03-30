use anchor_lang::prelude::*;

#[error_code]
pub enum ProcessorError {
    #[msg("You are not authorized to perform this operation")]
    UnauthorizedOwner,
    
    #[msg("Processor is paused")]
    ProcessorPaused,
    
    #[msg("Invalid authorization program ID")]
    InvalidAuthorizationProgram,
    
    #[msg("Unauthorized caller")]
    UnauthorizedCaller,
    
    #[msg("Priority queue is full")]
    QueueFull,
    
    #[msg("Priority queue is empty")]
    QueueEmpty,
    
    #[msg("Message batch has expired")]
    MessageBatchExpired,
    
    #[msg("Invalid message format")]
    InvalidMessageFormat,
    
    #[msg("Too many messages in batch")]
    TooManyMessages,
    
    #[msg("Execution failed")]
    ExecutionFailed,
    
    #[msg("Invalid subroutine type")]
    InvalidSubroutineType,
    
    #[msg("Invalid priority level")]
    InvalidPriorityLevel,
    
    #[msg("No pending callback found")]
    NoPendingCallback,
    
    #[msg("Callback already sent")]
    CallbackAlreadySent,
    
    #[msg("Message batch not found")]
    MessageBatchNotFound,
    
    #[msg("Cross-program invocation failed")]
    CpiError,
    
    #[msg("Execution limit exceeded")]
    ExecutionLimitExceeded,
} 