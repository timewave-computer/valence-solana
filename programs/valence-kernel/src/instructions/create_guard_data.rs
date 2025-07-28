// Create guard data instruction for valence-kernel
// Handles initialization of separate guard data accounts
use crate::{
    guards::SerializedGuard,
    state::GuardData,
};
use anchor_lang::prelude::*;

// ================================
// Instruction Handler
// ================================

/// Create a new guard data account with pre-compiled guard
/// 
/// This instruction creates a separate account to store compiled guard configuration.
/// The guard must be compiled client-side to avoid expensive on-chain computation.
pub fn create_guard_data(
    ctx: Context<CreateGuardData>,
    session: Pubkey,
    serialized_guard: SerializedGuard,
) -> Result<()> {
    // Validate the pre-compiled guard
    serialized_guard.validate()?;
    
    let guard_data = &mut ctx.accounts.guard_data;
    
    // Initialize guard data with compiled guard
    **guard_data = GuardData::create(session, serialized_guard);
    
    Ok(())
}

// ================================
// Account Context
// ================================

/// Account context for guard data creation
#[derive(Accounts)]
#[instruction(session: Pubkey, serialized_guard: SerializedGuard)]
pub struct CreateGuardData<'info> {
    /// The guard data account being created with exact sizing
    /// Space is dynamically calculated based on the compiled guard to avoid wasting rent
    #[account(
        init,
        payer = payer,
        space = GuardData::calculate_space_for_apu_program(&serialized_guard),
    )]
    pub guard_data: Account<'info, GuardData>,
    
    /// The payer for the account creation
    #[account(mut)]
    pub payer: Signer<'info>,
    
    /// System program for account creation
    pub system_program: Program<'info, System>,
}