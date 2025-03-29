use anchor_lang::prelude::*;
use crate::state::*;

pub fn handler(
    ctx: Context<Initialize>,
    valence_registry: Pubkey,
    processor_program_id: Pubkey,
) -> Result<()> {
    let state = &mut ctx.accounts.authorization_state;
    
    // Initialize the program state
    state.owner = ctx.accounts.owner.key();
    state.sub_owners = Vec::new();
    state.processor_program_id = processor_program_id;
    state.execution_counter = 0;
    state.valence_registry = valence_registry;
    state.bump = *ctx.bumps.get("authorization_state").unwrap();
    
    msg!("Authorization Program initialized!");
    msg!("Owner: {}", state.owner);
    msg!("Processor Program: {}", state.processor_program_id);
    msg!("Valence Registry: {}", state.valence_registry);
    
    Ok(())
} 