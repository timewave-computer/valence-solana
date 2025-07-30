// Comprehensive error system for valence-kernel operation processing
//
// The valence-kernel processes untrusted client operations and performs complex account
// borrowing, CPI calls, and session management. This error system provides structured,
// categorized error reporting that enables precise diagnosis of failures across all
// kernel subsystems. Each error includes detailed context and maps to specific failure
// modes in the batch execution engine.
//
// ERROR CATEGORIZATION: Errors are organized by functional domain (state, authorization,
// sessions, guards, accounts, etc.) with numeric ranges for easy identification and
// debugging. This structured approach helps developers quickly identify the source and
// nature of execution failures in complex operation batches.
//
// INTEGRATION: All kernel components use these standardized error types, ensuring
// consistent error handling and reporting across the entire execution pipeline.

use anchor_lang::prelude::*;

// ================================
// Rich Error System
// ================================

/// Core errors with detailed context
#[error_code]
pub enum KernelError {
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
    
    #[msg("Session has been invalidated")]
    SessionInactive, // 6205

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

    #[msg("Invalid guard manifest")]
    InvalidGuardManifest, // 6308

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
    
    #[msg("Borrowed account mismatch")]
    BorrowedAccountMismatch, // 6506
    
    #[msg("Missing required account")]
    MissingRequiredAccount, // 6507
    
    #[msg("Invalid account data")]
    InvalidAccountData, // 6508
    
    #[msg("Account index out of bounds")]
    AccountIndexOutOfBounds, // 6509
    
    #[msg("Invalid program index")]
    InvalidProgramIndex, // 6510
    
    #[msg("Unauthorized account")]
    UnauthorizedAccount, // 6511
    
    #[msg("Too many accounts registered")]
    TooManyAccounts, // 6512
    
    #[msg("Duplicate account registration")]
    DuplicateAccount, // 6513
    
    #[msg("Unregistered account")]
    UnregisteredAccount, // 6514
    
    #[msg("Account already exists")]
    AccountAlreadyExists, // 6515

    // ===== Performance Errors (6600-6699) =====
    #[msg("Compute budget exceeded")]
    ComputeBudgetExceeded, // 6600

    #[msg("CPI depth exceeded")]
    CrossProgramInvocationDepthExceeded, // 6601

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
    
    #[msg("Session reentrancy violation")]
    ReentrancyViolation, // 6802

    // ===== Namespace Errors (6900-6999) =====
    #[msg("Namespace path cannot be empty")]
    NamespaceEmptyPath, // 6900
    
    #[msg("Invalid namespace path format")]
    NamespaceInvalidPath, // 6901
    
    #[msg("Namespace segment cannot be empty")]
    NamespaceEmptySegment, // 6902
    
    #[msg("Namespace segment cannot contain slashes")]
    NamespaceInvalidSegment, // 6903
    
    #[msg("Insufficient privileges for operation")]
    NamespaceInsufficientPrivileges, // 6904
    
    #[msg("Namespace already exists")]
    NamespaceAlreadyExists, // 6905
    
    #[msg("Namespace not found")]
    NamespaceNotFound, // 6906
    
    #[msg("Cannot delete namespace with children")]
    NamespaceHasChildren, // 6907
    
    #[msg("State size exceeds maximum allowed")]
    NamespaceStateTooLarge, // 6908
}

