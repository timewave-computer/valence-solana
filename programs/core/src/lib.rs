/// Valence Protocol Core - Unified core protocol implementation
/// This is the main program entry point that orchestrates all valence functionality


use anchor_lang::prelude::*;

// Use a valid 32-byte program ID for development  
declare_id!("Va1enceCore11111111111111111111111111111111");

// Core program modules
pub mod capabilities;
pub mod config;
pub mod error;
pub mod events;
pub mod functions;
pub mod sessions;
pub mod state;

pub mod verification;

// Template module for standalone eval (consolidated from templates/mod.rs)
pub mod standalone_eval_template;

// Singleton execution modules (moved from separate programs)
pub mod processor;
pub mod scheduler;  
pub mod diff;

// Re-exports for core types (specific imports to avoid conflicts)
pub use capabilities::{ShardState, EvalConfig, ExecutionContext as CapabilityExecutionContext, ExecutionResult as CapabilityExecutionResult, PartialOrder, OrderingConstraint};
pub use error::{ValenceError, ProcessorError, SchedulerError, DiffError};
pub use state::*;
pub use events::*;



// Account definitions at program level - updated for shard-based structure
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = ShardState::SPACE,
        seeds = [b"shard_state"],
        bump
    )]
    pub shard_state: Account<'info, ShardState>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ExecuteCapability<'info> {
    #[account(mut)]
    pub shard_state: Account<'info, ShardState>,
    
    /// CHECK: Session account - verified by capability verification
    pub session: AccountInfo<'info>,
    
    #[account(mut)]
    pub executor: Signer<'info>,
}

#[derive(Accounts)]
pub struct Pause<'info> {
    #[account(
        mut,
        has_one = authority,
        seeds = [b"shard_state"],
        bump = shard_state.bump
    )]
    pub shard_state: Account<'info, ShardState>,
    
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct Resume<'info> {
    #[account(
        mut,
        has_one = authority,
        seeds = [b"shard_state"],
        bump = shard_state.bump
    )]
    pub shard_state: Account<'info, ShardState>,
    
    pub authority: Signer<'info>,
}

#[program]
pub mod core {
    use super::*;

    /// Initialize the core shard with eval capabilities
    pub fn initialize(
        ctx: Context<Initialize>,
        processor_program: Pubkey,
    ) -> Result<()> {
        msg!("Initializing shard with eval capabilities");
        ctx.accounts.shard_state.process_initialize(
            ctx.accounts.authority.key(),
            processor_program,
            ctx.bumps.shard_state,
        )?;
        Ok(())
    }

    /// Execute a capability using embedded eval logic
    pub fn execute_capability(
        ctx: Context<ExecuteCapability>,
        capability_id: String,
        input_data: Vec<u8>,
    ) -> Result<()> {
        let execution_context = CapabilityExecutionContext::new(
            capability_id.clone(),
            ctx.accounts.executor.key(),
            None, // No session for now
        ).with_input_data(input_data.clone());
        
        ctx.accounts.shard_state.process_execute_capability(
            capability_id,
            input_data,
            &execution_context,
        )?;
        Ok(())
    }

    /// Pause the shard
    pub fn pause(ctx: Context<Pause>) -> Result<()> {
        msg!("Pausing shard");
        ctx.accounts.shard_state.process_pause()?;
        Ok(())
    }

    /// Resume the shard
    pub fn resume(ctx: Context<Resume>) -> Result<()> {
        msg!("Resuming shard");
        ctx.accounts.shard_state.process_resume()?;
        Ok(())
    }
} 