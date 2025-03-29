use anchor_lang::prelude::*;

#[error_code]
pub enum BaseAccountError {
    #[msg("The authorization token is invalid")]
    InvalidAuthorizationToken,
    #[msg("The provided library is not approved")]
    LibraryNotApproved,
    #[msg("The instruction execution failed")]
    InstructionExecutionFailed,
    #[msg("The token account creation failed")]
    TokenAccountCreationFailed,
    #[msg("The token transfer failed")]
    TokenTransferFailed,
    #[msg("The operation requires owner authority")]
    UnauthorizedOwnerOperation,
    #[msg("The provided address does not match the expected base account")]
    InvalidBaseAccount,
    #[msg("The provided mint is not supported")]
    UnsupportedMint,
    #[msg("The token account already exists")]
    TokenAccountAlreadyExists,
    #[msg("The library registration failed")]
    LibraryRegistrationFailed,
    #[msg("The execution context is invalid")]
    InvalidExecutionContext,
} 