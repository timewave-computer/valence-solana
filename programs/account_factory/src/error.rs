use anchor_lang::prelude::*;

#[error_code]
pub enum AccountFactoryError {
    #[msg("The operation requires authority permissions")]
    UnauthorizedOperation,
    #[msg("Invalid template ID provided")]
    InvalidTemplateId,
    #[msg("Template with this ID already exists")]
    TemplateAlreadyExists,
    #[msg("Invalid template parameters")]
    InvalidTemplateParameters,
    #[msg("Template version mismatch")]
    TemplateVersionMismatch,
    #[msg("Invalid account type requested")]
    InvalidAccountType,
    #[msg("Invalid parameter for account creation")]
    InvalidAccountParameter,
    #[msg("Batch size exceeds maximum allowed")]
    BatchSizeExceeded,
    #[msg("Insufficient funds for account creation")]
    InsufficientFunds,
    #[msg("Account creation failed")]
    AccountCreationFailed,
    #[msg("Template not found")]
    TemplateNotFound,
    #[msg("Template is disabled")]
    TemplateDisabled,
    #[msg("The account seed is already in use")]
    AccountSeedInUse,
    #[msg("Template initialization failed")]
    TemplateInitializationFailed,
    #[msg("Cross-program invocation failed")]
    CpiInvocationFailed,
} 