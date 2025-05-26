// Standardized error handling for Authorization Program
// Following Valence Protocol error handling patterns

use anchor_lang::prelude::*;

/// Standardized error codes for the Authorization Program
/// Error codes are organized by category for better maintainability
#[error_code]
pub enum AuthorizationError {
    // === Authorization Management Errors (6000-6099) ===
    #[msg("Not authorized to perform this action")]
    NotAuthorized = 6000,
    
    #[msg("Authorization is not active")]
    AuthorizationInactive,
    
    #[msg("Authorization has expired")]
    AuthorizationExpired,
    
    #[msg("Authorization not yet valid")]
    AuthorizationNotYetValid,
    
    #[msg("Invalid authorization label")]
    InvalidAuthorizationLabel,
    
    #[msg("Authorization already exists")]
    AuthorizationAlreadyExists,
    
    #[msg("Authorization not found")]
    AuthorizationNotFound,
    
    // === Permission & Access Control Errors (6100-6199) ===
    #[msg("Unauthorized sender")]
    UnauthorizedSender = 6100,
    
    #[msg("Unauthorized callback")]
    UnauthorizedCallback,
    
    #[msg("Invalid permission type")]
    InvalidPermissionType,
    
    #[msg("User not in allowlist")]
    UserNotInAllowlist,
    
    #[msg("Allowlist is full")]
    AllowlistFull,
    
    #[msg("Invalid user address")]
    InvalidUserAddress,
    
    // === Execution Management Errors (6200-6299) ===
    #[msg("Maximum concurrent executions reached")]
    MaxConcurrentExecutionsReached = 6200,
    
    #[msg("Execution not found")]
    ExecutionNotFound,
    
    #[msg("Execution already completed")]
    ExecutionAlreadyCompleted,
    
    #[msg("Execution has expired")]
    ExecutionExpired,
    
    #[msg("Invalid execution state")]
    InvalidExecutionState,
    
    #[msg("Execution limit exceeded")]
    ExecutionLimitExceeded,
    
    // === Message Processing Errors (6300-6399) ===
    #[msg("Empty message batch")]
    EmptyMessageBatch = 6300,
    
    #[msg("Message too large")]
    MessageTooLarge,
    
    #[msg("Invalid message format")]
    InvalidMessageFormat,
    
    #[msg("Message batch too large")]
    MessageBatchTooLarge,
    
    #[msg("Invalid message type")]
    InvalidMessageType,
    
    #[msg("Message processing failed")]
    MessageProcessingFailed,
    
    #[msg("Payload too large")]
    PayloadTooLarge,
    
    // === ZK Proof & Verification Errors (6400-6499) ===
    #[msg("ZK program is inactive")]
    ZKProgramInactive = 6400,
    
    #[msg("ZK program not found")]
    ZKProgramNotFound,
    
    #[msg("Invalid ZK proof")]
    InvalidZKProof,
    
    #[msg("ZK proof verification failed")]
    ZKProofVerificationFailed,
    
    #[msg("Invalid verification key")]
    InvalidVerificationKey,
    
    #[msg("Verification key not found")]
    VerificationKeyNotFound,
    
    #[msg("ZK message replay detected")]
    ZKMessageReplay,
    
    #[msg("ZK message sequence violation")]
    ZKMessageSequenceViolation,
    
    #[msg("ZK message too old")]
    ZKMessageTooOld,
    
    #[msg("ZK message from future")]
    ZKMessageFromFuture,
    
    // === Program Integration Errors (6500-6599) ===
    #[msg("Invalid processor program ID")]
    InvalidProcessorProgramId = 6500,
    
    #[msg("Invalid registry program ID")]
    InvalidRegistryProgramId,
    
    #[msg("Invalid ZK verifier program ID")]
    InvalidZKVerifierProgramId,
    
    #[msg("Cross-program invocation failed")]
    CpiError,
    
    #[msg("Program account mismatch")]
    ProgramAccountMismatch,
    
    // === Validation & Parameter Errors (6600-6699) ===
    #[msg("Invalid parameters provided")]
    InvalidParameters = 6600,
    
    #[msg("Invalid timestamp")]
    InvalidTimestamp,
    
    #[msg("Invalid sequence number")]
    InvalidSequenceNumber,
    
    #[msg("Parameter out of range")]
    ParameterOutOfRange,
    
    #[msg("Required parameter missing")]
    RequiredParameterMissing,
    
    #[msg("Parameter validation failed")]
    ParameterValidationFailed,
    
    // === System & Resource Errors (6700-6799) ===
    #[msg("Insufficient funds")]
    InsufficientFunds = 6700,
    
    #[msg("Account creation failed")]
    AccountCreationFailed,
    
    #[msg("Account closure failed")]
    AccountClosureFailed,
    
    #[msg("Compute budget exceeded")]
    ComputeBudgetExceeded,
    
    #[msg("Transaction size limit exceeded")]
    TransactionSizeLimitExceeded,
    
    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,
    
    #[msg("Arithmetic underflow")]
    ArithmeticUnderflow,
    
    // === State Management Errors (6800-6899) ===
    #[msg("Invalid state transition")]
    InvalidStateTransition = 6800,
    
    #[msg("State corruption detected")]
    StateCorruption,
    
    #[msg("State synchronization failed")]
    StateSynchronizationFailed,
    
    #[msg("State rollback failed")]
    StateRollbackFailed,
    
    #[msg("Concurrent state modification")]
    ConcurrentStateModification,
}

/// Helper trait for standardized error handling
pub trait ErrorContext<T> {
    /// Add context to an error for better debugging
    fn with_context(self, context: &str) -> Result<T>;
    
    /// Convert to a specific error type with context
    fn to_error(self, error: AuthorizationError) -> Result<T>;
}

impl<T> ErrorContext<T> for Result<T> {
    fn with_context(self, context: &str) -> Result<T> {
        self.map_err(|e| {
            msg!("Error context: {}", context);
            e
        })
    }
    
    fn to_error(self, error: AuthorizationError) -> Result<T> {
        self.map_err(|_| error.into())
    }
}

/// Standardized error logging macro
#[macro_export]
macro_rules! log_error {
    ($error:expr, $context:expr) => {
        msg!("Authorization Error [{}]: {} - Context: {}", 
             $error as u32, 
             stringify!($error), 
             $context);
    };
    ($error:expr) => {
        msg!("Authorization Error [{}]: {}", 
             $error as u32, 
             stringify!($error));
    };
}

/// Validation helper functions
pub mod validation {
    use super::*;
    
    /// Validate that a value is within acceptable range
    pub fn validate_range<T: PartialOrd>(value: T, min: T, max: T) -> Result<()> {
        if value < min || value > max {
            return Err(AuthorizationError::ParameterOutOfRange.into());
        }
        Ok(())
    }
    
    /// Validate that a string is not empty and within length limits
    pub fn validate_string(s: &str, max_len: usize) -> Result<()> {
        if s.is_empty() {
            return Err(AuthorizationError::RequiredParameterMissing.into());
        }
        if s.len() > max_len {
            return Err(AuthorizationError::ParameterOutOfRange.into());
        }
        Ok(())
    }
    
    /// Validate timestamp is reasonable (not too far in past/future)
    pub fn validate_timestamp(timestamp: i64, max_age_seconds: i64) -> Result<()> {
        let current_time = anchor_lang::solana_program::clock::Clock::get()?.unix_timestamp;
        let age = current_time - timestamp;
        
        if age > max_age_seconds {
            return Err(AuthorizationError::ZKMessageTooOld.into());
        }
        
        if age < -300 { // Allow 5 minute clock skew
            return Err(AuthorizationError::ZKMessageFromFuture.into());
        }
        
        Ok(())
    }
} 