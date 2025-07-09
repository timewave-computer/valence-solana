// Processor instruction handlers

use anchor_lang::prelude::*;
use crate::{processor::ProcessorState, ValenceError};

/// Initialize the processor singleton
pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
    let processor_state = &mut ctx.accounts.processor_state;
    processor_state.authority = ctx.accounts.authority.key();
    processor_state.is_paused = false;
    processor_state.total_executions = 0;
    processor_state.bump = ctx.bumps.processor_state;
    
    msg!("Processor singleton initialized");
    Ok(())
}

/// Process a capability execution
pub fn process_capability(
    ctx: Context<ProcessCapability>,
    capability_id: String,
    input_data: Vec<u8>,
) -> Result<()> {
    let processor_state = &mut ctx.accounts.processor_state;
    
    // Check if processor is paused
    require!(!processor_state.is_paused, ValenceError::ProcessorPaused);
    
    // Basic capability processing logic (to be expanded)
    processor_state.total_executions += 1;
    
    msg!("Processed capability: {} with {} bytes of input", capability_id, input_data.len());
    Ok(())
}

/// Pause the processor
pub fn pause(ctx: Context<Pause>) -> Result<()> {
    let processor_state = &mut ctx.accounts.processor_state;
    processor_state.is_paused = true;
    
    msg!("Processor paused");
    Ok(())
}

/// Resume the processor
pub fn resume(ctx: Context<Resume>) -> Result<()> {
    let processor_state = &mut ctx.accounts.processor_state;
    processor_state.is_paused = false;
    
    msg!("Processor resumed");
    Ok(())
}

// Account contexts

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        space = ProcessorState::SPACE,
        seeds = [b"processor_state"],
        bump
    )]
    pub processor_state: Account<'info, ProcessorState>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ProcessCapability<'info> {
    #[account(mut)]
    pub processor_state: Account<'info, ProcessorState>,
    
    pub caller: Signer<'info>,
}

#[derive(Accounts)]
pub struct Pause<'info> {
    #[account(
        mut,
        has_one = authority,
        seeds = [b"processor_state"],
        bump = processor_state.bump
    )]
    pub processor_state: Account<'info, ProcessorState>,
    
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct Resume<'info> {
    #[account(
        mut,
        has_one = authority,
        seeds = [b"processor_state"],
        bump = processor_state.bump
    )]
    pub processor_state: Account<'info, ProcessorState>,
    
    pub authority: Signer<'info>,
} 