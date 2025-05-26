// Update verification key instruction for ZK Proof Verifier Program

use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::VerifierError;

pub fn handler(
    ctx: Context<UpdateVerificationKey>,
    verification_key: Vec<u8>,
    key_type: VerificationKeyType,
) -> Result<()> {
    let verification_key_account = &mut ctx.accounts.verification_key_account;
    
    // Validate verification key
    if verification_key.is_empty() || verification_key.len() > 4096 {
        return Err(VerifierError::InvalidVerificationKey.into());
    }
    
    // Update the verification key data
    verification_key_account.key_data = verification_key;
    
    // Update the key type
    verification_key_account.key_type = key_type;
    
    // Update timestamp
    verification_key_account.updated_at = Clock::get()?.unix_timestamp;
    
    msg!(
        "Verification key updated for program: {}, type: {:?}",
        verification_key_account.program_id,
        verification_key_account.key_type
    );
    
    Ok(())
} 