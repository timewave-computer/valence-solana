use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::ProcessorError;
use crate::queue::{QueueManager, DEFAULT_QUEUE_CAPACITY};

pub fn handler(
    ctx: Context<Initialize>,
    authorization_program_id: Pubkey,
) -> Result<()> {
    // Get the processor state account
    let processor_state = &mut ctx.accounts.processor_state;
    
    // Set the authorization program ID
    processor_state.authorization_program_id = authorization_program_id;
    
    // Set the owner
    processor_state.owner = ctx.accounts.owner.key();
    
    // Initialize the queues
    processor_state.high_priority_queue = QueueState::new(DEFAULT_QUEUE_CAPACITY);
    processor_state.medium_priority_queue = QueueState::new(DEFAULT_QUEUE_CAPACITY);
    processor_state.low_priority_queue = QueueState::new(DEFAULT_QUEUE_CAPACITY);
    
    // Initialize stats
    processor_state.total_executions = 0;
    processor_state.successful_executions = 0;
    processor_state.failed_executions = 0;
    processor_state.last_execution_time = 0;
    
    // Initialize state to not paused
    processor_state.is_paused = false;
    
    // Store the bump seed
    processor_state.bump = *ctx.bumps.get("processor_state").unwrap();
    
    // Log the initialization
    msg!(
        "Processor initialized with authorization program: {}, owner: {}",
        processor_state.authorization_program_id,
        processor_state.owner
    );
    
    Ok(())
} 