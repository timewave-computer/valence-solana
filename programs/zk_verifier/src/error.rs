// Error types for ZK Verifier Program

use anchor_lang::prelude::*;

#[error_code]
pub enum VerifierError {
    #[msg("Verification key is not active")]
    VerificationKeyInactive,
    
    #[msg("Verification key not found")]
    VerificationKeyNotFound,
    
    #[msg("Invalid verification key")]
    InvalidVerificationKey,
    
    #[msg("Invalid proof data")]
    InvalidProof,
    
    #[msg("Invalid parameters")]
    InvalidParameters,
    
    #[msg("Proof verification failed")]
    ProofVerificationFailed,
    
    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,
    
    #[msg("Arithmetic underflow")]
    ArithmeticUnderflow,
} 