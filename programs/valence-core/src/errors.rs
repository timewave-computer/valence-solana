use anchor_lang::prelude::*;

// ================================
// Rich Error System
// ================================

/// Core errors with detailed context
#[error_code]
pub enum ValenceError {
    // ===== State Errors (6000-6099) =====
    #[msg("State expired")]
    StateExpired, // 6000

    #[msg("State not active")]
    StateNotActive, // 6001

    #[msg("State locked")]
    StateLocked, // 6002

    #[msg("Invalid state transition")]
    InvalidStateTransition, // 6003

    #[msg("State already exists")]
    StateAlreadyExists, // 6004

    // ===== Authorization Errors (6100-6199) =====
    #[msg("Unauthorized")]
    Unauthorized, // 6100

    // Removed: AuthorizationExpired - now handled by Guard::Expiration
    #[msg("Insufficient permissions")]
    InsufficientPermissions, // 6102

    // Removed: DelegateNotAuthorized - now handled by guards
    #[msg("Usage limit exceeded")]
    UsageLimitExceeded, // 6104

    // ===== Session Errors (6200-6299) =====
    #[msg("Session expired")]
    SessionExpired, // 6200

    #[msg("Session state not found")]
    SessionStateNotFound, // 6201

    #[msg("Max bound states exceeded")]
    MaxBoundStatesExceeded, // 6202

    #[msg("Session paused")]
    SessionPaused, // 6203

    #[msg("Invalid session config")]
    InvalidSessionConfig, // 6204

    // ===== Guard Errors (6300-6399) =====
    #[msg("Guard verification failed")]
    GuardFailed, // 6300

    #[msg("External guard required")]
    ExternalGuardRequired, // 6301

    #[msg("Invalid guard program")]
    InvalidGuardProgram, // 6302

    // Removed: InvalidAuthorizationState - no longer using authorization state
    #[msg("Guard depth exceeded maximum")]
    GuardDepthExceeded, // 6304

    #[msg("External guard did not return data")]
    ExternalGuardNoReturnData, // 6305

    #[msg("External guard returned invalid data")]
    ExternalGuardInvalidReturn, // 6306

    #[msg("Guard data too large")]
    GuardDataTooLarge, // 6307

    // ===== Account Errors (6400-6499) =====
    #[msg("Account too small")]
    AccountDataTooSmall, // 6400

    #[msg("Account not writable")]
    AccountNotWritable, // 6401

    #[msg("Account owner mismatch")]
    AccountOwnerMismatch, // 6402

    #[msg("Invalid PDA")]
    InvalidPDA, // 6403

    #[msg("Account already initialized")]
    AccountAlreadyInitialized, // 6404

    // ===== Data Errors (6500-6599) =====
    #[msg("Invalid version")]
    InvalidVersion, // 6500

    #[msg("Invalid reserved data")]
    InvalidReservedData, // 6501

    #[msg("Invalid parameters")]
    InvalidParameters, // 6502
    
    #[msg("Account is already borrowed")]
    AccountAlreadyBorrowed, // 6503
    
    #[msg("Borrow capacity exceeded")]
    BorrowCapacityExceeded, // 6504
    
    #[msg("Account not borrowed")]
    AccountNotBorrowed, // 6505

    // ===== Performance Errors (6600-6699) =====
    #[msg("Compute budget exceeded")]
    ComputeBudgetExceeded, // 6600

    #[msg("CPI depth exceeded")]
    CpiDepthExceeded, // 6601

    #[msg("Transaction too large")]
    TransactionTooLarge, // 6602
    
    // ===== CPI Security Errors (6700-6799) =====
    #[msg("Program not in CPI allowlist")]
    ProgramNotAllowed, // 6700
    
    #[msg("Program already in allowlist")]
    ProgramAlreadyAllowed, // 6701
    
    #[msg("CPI allowlist is full")]
    AllowlistFull, // 6702
    
    // ===== Transaction Security Errors (6800-6899) =====
    #[msg("Transaction-wide reentrancy detected")]
    ReentrancyGuardViolation, // 6800
    
    #[msg("Invalid transaction signature")]
    InvalidTransaction, // 6801
}

// ================================
// Error Context Builder
// ================================

/// Generic error macro with context logging
#[macro_export]
macro_rules! err_with_context {
    ($error:expr, $msg:literal $(, $arg:expr)*) => {{
        anchor_lang::prelude::msg!($msg $(, $arg)*);
        $error
    }};
}

// Error helpers
pub trait ErrorContext {
    fn context(self, msg: &str) -> Self;
}

impl<T> ErrorContext for Result<T> {
    fn context(self, msg: &str) -> Self {
        self.inspect_err(|_| msg!("Context: {}", msg))
    }
}

#[cfg(debug_assertions)]
pub fn log_error_details(error: &ValenceError, context: &str) {
    msg!("ERROR: {:?} - {}", error, context);
}

#[cfg(not(debug_assertions))]
pub fn log_error_details(_: &ValenceError, _: &str) {}
