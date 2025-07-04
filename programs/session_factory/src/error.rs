use anchor_lang::prelude::*;

#[error_code]
pub enum SessionFactoryError {
    #[msg("Unauthorized: Only the factory owner can perform this action")]
    Unauthorized,
    
    #[msg("Too many namespaces specified")]
    TooManyNamespaces,
    
    #[msg("Invalid eval program ID")]
    InvalidEvalProgram,
    
    #[msg("Session already exists")]
    SessionAlreadyExists,
} 