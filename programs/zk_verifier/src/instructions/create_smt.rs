// Create SMT instruction for ZK Proof Verifier Program

use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::VerifierError;

pub fn handler(
    ctx: Context<CreateSMT>,
    height: u8,
) -> Result<()> {
    let smt_state = &mut ctx.accounts.smt_state;
    
    // Validate height (typically 256 for 32-byte keys, but u8 max is 255)
    if height == 0 || height > 255 {
        return Err(VerifierError::InvalidParameters.into());
    }
    
    // Initialize SMT state
    smt_state.owner = ctx.accounts.owner.key();
    smt_state.root = [0u8; 32]; // Empty tree root
    smt_state.height = height;
    smt_state.leaf_count = 0;
    smt_state.last_updated = Clock::get()?.unix_timestamp;
    smt_state.is_frozen = false;
    smt_state.bump = ctx.bumps.smt_state;
    
    msg!("Created SMT with height {} and owner {}", height, smt_state.owner);
    
    Ok(())
}

#[derive(Accounts)]
#[instruction(height: u8)]
pub struct CreateSMT<'info> {
    /// The SMT state account
    #[account(
        init,
        payer = owner,
        space = 8 + SMTState::SPACE,
        seeds = [b"smt_state".as_ref(), owner.key().as_ref()],
        bump
    )]
    pub smt_state: Account<'info, SMTState>,
    
    /// The account paying for the initialization
    #[account(mut)]
    pub owner: Signer<'info>,
    
    /// System program for creating accounts
    pub system_program: Program<'info, System>,
} 