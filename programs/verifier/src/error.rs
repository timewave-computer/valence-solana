//! Verifier errors

use anchor_lang::prelude::*;

#[error_code]
pub enum VerificationError {
    #[msg("Invalid verifier label")]
    InvalidLabel,
    
    #[msg("Invalid program ID")]
    InvalidProgram,
    
    #[msg("Verifier label mismatch")]
    LabelMismatch,
    
    #[msg("Unauthorized access")]
    Unauthorized,
    
    #[msg("Verifier not found")]
    VerifierNotFound,
    
    #[msg("Verification failed")]
    VerificationFailed,
    
    #[msg("Invalid predicate data")]
    InvalidPredicateData,
}