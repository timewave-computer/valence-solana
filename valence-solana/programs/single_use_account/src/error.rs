use anchor_lang::prelude::*;

#[error_code]
pub enum SingleUseAccountError {
    #[msg("The operation requires owner authority")]
    UnauthorizedOwnerOperation,
    #[msg("The account has already been used")]
    AccountAlreadyUsed,
    #[msg("The token account has remaining balance after execution")]
    RemainingBalanceAfterExecution,
    #[msg("The destination address does not match required destination")]
    InvalidDestination,
    #[msg("The account has not expired yet")]
    AccountNotExpired,
    #[msg("The account has no expiration time set")]
    NoExpirationTime,
    #[msg("The expiration time is in the past")]
    ExpirationInPast,
    #[msg("The provided library is not approved")]
    LibraryNotApproved,
    #[msg("The instruction execution failed")]
    ExecutionFailed,
    #[msg("The token transfer failed")]
    TokenTransferFailed,
    #[msg("The token account verification failed")]
    TokenAccountVerificationFailed,
    #[msg("The token account is not empty")]
    TokenAccountNotEmpty,
    #[msg("The provided address does not match the expected single-use account")]
    InvalidSingleUseAccount,
} 