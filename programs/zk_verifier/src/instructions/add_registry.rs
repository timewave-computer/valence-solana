// Add registry instruction for ZK Verifier Program

use anchor_lang::prelude::*;
use crate::state::*;

pub fn handler(
    ctx: Context<crate::AddRegistry>,
    registry_id: u64,
    vk_hash: [u8; 32],
    key_type: VerificationKeyType,
) -> Result<()> {
    let verification_key = &mut ctx.accounts.verification_key;
    let verifier_state = &mut ctx.accounts.verifier_state;
    
    // Set the verification key data (like Solidity programVKs[msg.sender][registry])
    verification_key.program_id = ctx.accounts.owner.key();
    verification_key.registry_id = registry_id;
    verification_key.vk_hash = vk_hash;
    verification_key.key_type = key_type;
    verification_key.is_active = true;
    verification_key.bump = ctx.bumps.verification_key;
    
    // Increment total keys counter
    verifier_state.total_keys = verifier_state.total_keys.checked_add(1)
        .ok_or(error!(crate::error::VerifierError::ArithmeticOverflow))?;
    
    msg!("Added verification key for program: {}, registry: {}", 
         verification_key.program_id, registry_id);
    
    Ok(())
}

 