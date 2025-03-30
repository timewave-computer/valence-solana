use anchor_lang::prelude::*;

#[error_code]
pub enum TokenTransferError {
    #[msg("The operation requires authority permissions")]
    UnauthorizedOperation,
    #[msg("Insufficient token balance for transfer")]
    InsufficientFunds,
    #[msg("The destination account has no delegate set")]
    DelegateNotSet,
    #[msg("The delegate does not match the expected delegate")]
    InvalidDelegate,
    #[msg("Insufficient delegated token amount for transfer")]
    InsufficientDelegatedAmount,
    #[msg("The mint is not authorized for transfers")]
    UnauthorizedMint,
    #[msg("The recipient is not authorized to receive tokens")]
    UnauthorizedRecipient,
    #[msg("The source account is not authorized to send tokens")]
    UnauthorizedSource,
    #[msg("The transfer amount exceeds the maximum allowed")]
    TransferAmountExceedsLimit,
    #[msg("Transfer amount must be greater than zero")]
    InvalidAmount,
    #[msg("The processor program does not match the expected program")]
    InvalidProcessorProgram,
    #[msg("The library is currently inactive")]
    LibraryInactive,
    #[msg("The mint of source and destination accounts must match")]
    MintMismatch,
    #[msg("Arithmetic operation overflowed")]
    ArithmeticOverflow,
    #[msg("A fee collector account is required for fees")]
    FeeCollectorRequired,
    #[msg("The fee exceeds the maximum allowed")]
    FeeExceedsLimit,
    #[msg("The slippage exceeds the maximum allowed")]
    SlippageExceedsLimit,
    #[msg("Account mismatch")]
    AccountMismatch,
    #[msg("The token account owner does not match the expected owner")]
    OwnerMismatch,
    #[msg("The batch transfer contains too many transfers")]
    BatchSizeExceeded,
    
    #[msg("Batch transfers are disabled for this library")]
    BatchTransfersDisabled,
    
    // New variants added to fix missing error types
    #[msg("The instruction data cannot be empty")]
    EmptyInstructionData,
    #[msg("The allowlist is empty")]
    EmptyAllowlist,
    #[msg("Invalid batch size")]
    InvalidBatchSize,
    #[msg("The account has no write permission")]
    NoWritePermission,
    #[msg("The account cannot be closed")]
    CannotCloseAccount,
    #[msg("Invalid account data")]
    InvalidAccountData,
    #[msg("The account is not owned by the expected program")]
    InvalidAccountOwner,
    #[msg("The account is not initialized")]
    UninitializedAccount,
    #[msg("Invalid instruction")]
    InvalidInstruction,
    #[msg("Account already initialized")]
    AccountAlreadyInitialized,
    #[msg("Account cannot be initialized")]
    CannotInitializeAccount,
    #[msg("Account not initialized")]
    AccountNotInitialized,
    #[msg("Operation not allowed")]
    OperationNotAllowed,
    #[msg("Insufficient SOL balance for transfer")]
    InsufficientSolBalance,
    #[msg("Insufficient rent for account")]
    InsufficientRent,
    #[msg("Rent exempt account required")]
    RentExemptRequired,
    #[msg("Invalid program ID")]
    InvalidProgramId,
    #[msg("Program error")]
    ProgramError,
    #[msg("Signature verification failed")]
    SignatureVerificationFailed,
    #[msg("Invalid seed")]
    InvalidSeed,
    #[msg("Invalid nonce")]
    InvalidNonce,
    #[msg("Expired nonce")]
    ExpiredNonce,
    #[msg("Unauthorized signer")]
    UnauthorizedSigner,
} 