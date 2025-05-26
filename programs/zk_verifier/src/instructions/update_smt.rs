// Update SMT instruction for ZK Proof Verifier Program

use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::VerifierError;

pub fn handler(
    ctx: Context<UpdateSMT>,
    key: [u8; 32],
    value: [u8; 32],
) -> Result<()> {
    let smt_state = &mut ctx.accounts.smt_state;
    let smt_leaf = &mut ctx.accounts.smt_leaf;
    
    // Check if SMT is frozen
    if smt_state.is_frozen {
        return Err(VerifierError::InvalidParameters.into());
    }
    
    // Update leaf
    smt_leaf.key = key;
    smt_leaf.value = value;
    smt_leaf.leaf_hash = smt_leaf.calculate_hash();
    smt_leaf.timestamp = Clock::get()?.unix_timestamp;
    smt_leaf.bump = ctx.bumps.smt_leaf;
    
    // Update SMT state
    smt_state.leaf_count = smt_state.leaf_count.checked_add(1)
        .ok_or(VerifierError::InvalidParameters)?;
    smt_state.last_updated = smt_leaf.timestamp;
    
    // TODO: Update root hash based on the new leaf
    // This would require implementing the full SMT update algorithm
    msg!("Updated SMT leaf with key: {:?}", key);
    
    Ok(())
}

#[derive(Accounts)]
#[instruction(key: [u8; 32], value: [u8; 32])]
pub struct UpdateSMT<'info> {
    /// The SMT state account
    #[account(
        mut,
        seeds = [b"smt_state".as_ref(), smt_state.owner.as_ref()],
        bump = smt_state.bump,
        constraint = smt_state.owner == owner.key() @ VerifierError::NotAuthorized,
    )]
    pub smt_state: Account<'info, SMTState>,
    
    /// The SMT leaf account
    #[account(
        init_if_needed,
        payer = owner,
        space = 8 + SMTLeaf::SPACE,
        seeds = [b"smt_leaf".as_ref(), smt_state.key().as_ref(), key.as_ref()],
        bump
    )]
    pub smt_leaf: Account<'info, SMTLeaf>,
    
    /// The owner of the SMT
    #[account(mut)]
    pub owner: Signer<'info>,
    
    /// System program for creating accounts
    pub system_program: Program<'info, System>,
} 