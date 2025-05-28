use anchor_lang::prelude::*;
use crate::state::*;

pub fn handler(ctx: Context<PauseProcessor>) -> Result<()> {
    // Get processor state
    let processor_state = &mut ctx.accounts.processor_state;
    
    // Check if already paused
    if processor_state.is_paused {
        msg!("Processor is already paused");
        return Ok(());
    }
    
    // Set to paused
    processor_state.is_paused = true;
    
    // Log the action
    msg!("Processor paused by owner: {}", ctx.accounts.owner.key());
    
    Ok(())
} 