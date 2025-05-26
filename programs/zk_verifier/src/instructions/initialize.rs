// Initialize instruction for ZK Proof Verifier Program

use anchor_lang::prelude::*;
use crate::state::*;

pub fn handler(ctx: Context<Initialize>) -> Result<()> {
    let verifier_state = &mut ctx.accounts.verifier_state;
    
    // Set the owner
    verifier_state.owner = ctx.accounts.owner.key();
    
    // Initialize counters
    verifier_state.total_keys = 0;
    verifier_state.successful_verifications = 0;
    verifier_state.failed_verifications = 0;
    
    // Start unpaused
    verifier_state.is_paused = false;
    
    // Store the bump seed
    verifier_state.bump = ctx.bumps.verifier_state;
    
    msg!("ZK Verifier initialized with owner: {}", verifier_state.owner);
    
    Ok(())
} 