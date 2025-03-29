use anchor_lang::prelude::*;

#[error_code]
pub enum TokenTransferError {
    #[msg("The operation requires authority permissions")]
    UnauthorizedOperation,
    #[msg("Insufficient token balance for transfer")]
    InsufficientBalance,
    #[msg("Invalid token account provided")]
    InvalidTokenAccount,
    #[msg("The provided source doesn't match the token account owner")]
    SourceOwnerMismatch,
    #[msg("The provided destination doesn't match the token account owner")]
    DestinationOwnerMismatch,
    #[msg("The account doesn't have the required approval")]
    ApprovalMissing,
    #[msg("The account has been frozen")]
    AccountFrozen,
    #[msg("The provided amount is zero or negative")]
    InvalidAmount,
    #[msg("Slippage tolerance exceeded")]
    SlippageTolerance,
    #[msg("The provided recipient is not on the allowlist")]
    RecipientNotAllowed,
    #[msg("The batch transfer contains too many transfers")]
    BatchSizeExceeded,
    #[msg("The batch transfer contains duplicate token accounts")]
    DuplicateTokenAccounts,
    #[msg("The source and destination token mints don't match")]
    MintMismatch,
    #[msg("The operation requires the associated token account to be created first")]
    TokenAccountNotCreated,
    #[msg("Cross-program invocation failed")]
    CpiInvocationFailed,
    #[msg("The token account belongs to a different mint")]
    WrongTokenMint,
} 