// Register verification key instruction for ZK Proof Verifier Program

use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::VerifierError;

pub fn handler(
    ctx: Context<RegisterVerificationKey>,
    program_id: Pubkey,
    verification_key: Vec<u8>,
    key_type: VerificationKeyType,
) -> Result<()> {
    let verifier_state = &mut ctx.accounts.verifier_state;
    let verification_key_account = &mut ctx.accounts.verification_key_account;
    
    // Validate verification key
    if verification_key.is_empty() || verification_key.len() > 4096 {
        return Err(VerifierError::InvalidVerificationKey.into());
    }
    
    // Set the program ID
    verification_key_account.program_id = program_id;
    
    // Set the verification key data
    verification_key_account.key_data = verification_key;
    
    // Set the key type
    verification_key_account.key_type = key_type;
    
    // Set as active
    verification_key_account.is_active = true;
    
    // Set timestamps
    let current_time = Clock::get()?.unix_timestamp;
    verification_key_account.registered_at = current_time;
    verification_key_account.updated_at = current_time;
    
    // Initialize verification count
    verification_key_account.verification_count = 0;
    
    // Store the bump seed
    verification_key_account.bump = ctx.bumps.verification_key_account;
    
    // Update verifier state
    verifier_state.total_keys = verifier_state.total_keys
        .checked_add(1)
        .ok_or(VerifierError::InvalidParameters)?;
    
    msg!(
        "Verification key registered for program: {}, type: {:?}",
        program_id,
        verification_key_account.key_type
    );
    
    Ok(())
} 