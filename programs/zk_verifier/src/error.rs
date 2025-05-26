// Error types for ZK Proof Verifier Program

use anchor_lang::prelude::*;

#[error_code]
pub enum VerifierError {
    #[msg("Not authorized to perform this action")]
    NotAuthorized,
    
    #[msg("Verifier is currently paused")]
    VerifierPaused,
    
    #[msg("Verification key is inactive")]
    VerificationKeyInactive,
    
    #[msg("Invalid verification key format")]
    InvalidVerificationKey,
    
    #[msg("Invalid proof format")]
    InvalidProof,
    
    #[msg("Invalid public inputs")]
    InvalidPublicInputs,
    
    #[msg("Proof verification failed")]
    ProofVerificationFailed,
    
    #[msg("Verification key not found")]
    VerificationKeyNotFound,
    
    #[msg("Verification key already exists")]
    VerificationKeyAlreadyExists,
    
    #[msg("Unsupported verification key type")]
    UnsupportedKeyType,
    
    #[msg("Compute budget exceeded")]
    ComputeBudgetExceeded,
    
    #[msg("Invalid parameters")]
    InvalidParameters,
    
    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,
} 