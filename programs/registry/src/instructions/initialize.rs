use anchor_lang::prelude::*;
use crate::state::*;

pub fn handler(
    ctx: Context<Initialize>,
    authorization_program_id: Pubkey,
    account_factory: Pubkey,
) -> Result<()> {
    // Get the registry state account
    let registry_state = &mut ctx.accounts.registry_state;
    
    // Set the owner
    registry_state.owner = ctx.accounts.owner.key();
    
    // Set the authorization program ID
    registry_state.authorization_program_id = authorization_program_id;
    
    // Set the account factory
    registry_state.account_factory = account_factory;
    
    // Store the bump seed
    registry_state.bump = ctx.bumps.registry_state;
    
    // Log the initialization
    msg!(
        "Registry initialized with owner: {}, authorization program: {}, account factory: {}",
        registry_state.owner,
        registry_state.authorization_program_id,
        registry_state.account_factory
    );
    
    Ok(())
} 