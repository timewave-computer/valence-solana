// Verify instruction for ZK Verifier Program

use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::VerifierError;

pub fn handler(
    ctx: Context<crate::Verify>,
    registry_id: u64,
    proof: Vec<u8>,
    message: Vec<u8>,
) -> Result<bool> {
    let verification_key = &ctx.accounts.verification_key;
    
    // Check if verification key is active
    if !verification_key.is_active {
        return Err(VerifierError::VerificationKeyInactive.into());
    }
    
    // Basic validation like Solidity implementation
    if proof.is_empty() {
        return Err(VerifierError::InvalidProof.into());
    }
    
    if message.is_empty() {
        return Err(VerifierError::InvalidParameters.into());
    }
    
    // Simple verification logic based on key type
    let verification_result = match verification_key.key_type {
        VerificationKeyType::SP1 => verify_sp1(&verification_key.vk_hash, &proof, &message)?,
        VerificationKeyType::Groth16 => verify_groth16(&verification_key.vk_hash, &proof, &message)?,
        VerificationKeyType::PLONK => verify_plonk(&verification_key.vk_hash, &proof, &message)?,
    };
    
    msg!("Verification result for registry {}: {}", registry_id, verification_result);
    
    Ok(verification_result)
}



/// Simple SP1 verification (placeholder like Solidity)
fn verify_sp1(vk_hash: &[u8; 32], proof: &[u8], message: &[u8]) -> Result<bool> {
    // In production, this would call actual SP1 verifier
    // For now, just basic validation
    if proof.len() < 64 || message.is_empty() || vk_hash.iter().all(|&x| x == 0) {
        return Ok(false);
    }
    Ok(true)
}

/// Simple Groth16 verification (placeholder)
fn verify_groth16(vk_hash: &[u8; 32], proof: &[u8], message: &[u8]) -> Result<bool> {
    // Groth16 proofs are typically 192 bytes
    if proof.len() != 192 || message.is_empty() || vk_hash.iter().all(|&x| x == 0) {
        return Ok(false);
    }
    Ok(true)
}

/// Simple PLONK verification (placeholder)
fn verify_plonk(vk_hash: &[u8; 32], proof: &[u8], message: &[u8]) -> Result<bool> {
    // PLONK proofs are variable size but typically larger
    if proof.len() < 384 || message.is_empty() || vk_hash.iter().all(|&x| x == 0) {
        return Ok(false);
    }
    Ok(true)
}

 