//! Registry errors

use anchor_lang::prelude::*;

#[error_code]
pub enum RegistryError {
    #[msg("Invalid program ID")]
    InvalidProgram,
    
    #[msg("Function hash mismatch")]
    HashMismatch,
    
    #[msg("Unauthorized access")]
    Unauthorized,
    
    #[msg("Invalid hash")]
    InvalidHash,
    
    #[msg("Function already registered")]
    FunctionAlreadyRegistered,
    
    #[msg("Function not found")]
    FunctionNotFound,
    
    #[msg("Invalid capability")]
    InvalidCapability,
    
    #[msg("Duplicate capability")]
    DuplicateCapability,
}