use anchor_lang::prelude::*;

#[error_code]
pub enum BaseAccountError {
    #[msg("Not authorized to perform this action")]
    Unauthorized,
    
    #[msg("The provided library is not approved")]
    LibraryNotApproved,
    
    #[msg("The library is already approved")]
    LibraryAlreadyApproved,
    
    #[msg("The provided token account already exists")]
    TokenAccountAlreadyExists,
    
    #[msg("Mint not supported")]
    MintNotSupported,
    
    #[msg("Token account creation failed")]
    TokenAccountCreationFailed,
    
    #[msg("Token account closure failed")]
    TokenAccountClosureFailed,
    
    #[msg("Instruction execution failed")]
    InstructionExecutionFailed,
    
    #[msg("The approval nonce has expired")]
    ApprovalNonceExpired,
    
    #[msg("The approval nonce has already been used")]
    ApprovalNonceUsed,
    
    #[msg("Invalid vault authority")]
    InvalidVaultAuthority,
    
    #[msg("Invalid approval nonce")]
    InvalidApprovalNonce,
    
    #[msg("Token account not found")]
    TokenAccountNotFound,
    
    #[msg("Invalid owner")]
    InvalidOwner,
    
    #[msg("Invalid parameters provided")]
    InvalidParameters,
} 