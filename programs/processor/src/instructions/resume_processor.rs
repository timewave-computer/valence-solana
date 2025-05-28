use anchor_lang::prelude::*;
use crate::state::*;

pub fn handler(ctx: Context<ResumeProcessor>) -> Result<()> {
    // Get processor state
    let processor_state = &mut ctx.accounts.processor_state;
    
    // Check if already running
    if !processor_state.is_paused {
        msg!("Processor is already running");
        return Ok(());
    }
    
    // Set to running
    processor_state.is_paused = false;
    
    // Log the action
    msg!("Processor resumed by owner: {}", ctx.accounts.owner.key());
    
    Ok(())
} 