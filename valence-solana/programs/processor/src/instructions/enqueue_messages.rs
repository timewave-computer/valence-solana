use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::ProcessorError;
use crate::queue::{QueueManager, priority_to_string};

pub fn handler(
    ctx: Context<EnqueueMessages>,
    execution_id: u64,
    priority: u8,
    subroutine_type: u8,
    messages: Vec<ProcessorMessage>,
) -> Result<()> {
    // Check message count
    if messages.is_empty() {
        return Err(error!(ProcessorError::InvalidMessageFormat));
    }
    
    if messages.len() > 20 {
        return Err(error!(ProcessorError::TooManyMessages));
    }
    
    // Convert priority and subroutine type
    let priority_enum = Priority::from(priority);
    let subroutine_type_enum = SubroutineType::from(subroutine_type);
    
    // Get processor state
    let processor_state = &mut ctx.accounts.processor_state;
    
    // Create queue manager
    let mut queue_manager = QueueManager::new(
        &mut processor_state.high_priority_queue,
        &mut processor_state.medium_priority_queue,
        &mut processor_state.low_priority_queue,
    );
    
    // Enqueue to the appropriate queue
    let index = queue_manager.enqueue(&priority_enum)?;
    
    // Initialize message batch
    let message_batch = &mut ctx.accounts.message_batch;
    message_batch.execution_id = execution_id;
    message_batch.messages = messages;
    message_batch.subroutine_type = subroutine_type_enum;
    message_batch.priority = priority_enum;
    message_batch.caller = ctx.accounts.caller.key();
    message_batch.callback_address = ctx.accounts.callback_address.key();
    
    // Set expiration time (if needed, could be 10 minutes from now)
    message_batch.expiration_time = Some(Clock::get()?.unix_timestamp + 600);
    
    // Set creation timestamp
    message_batch.created_at = Clock::get()?.unix_timestamp;
    
    // Store the bump seed
    message_batch.bump = *ctx.bumps.get("message_batch").unwrap();
    
    // Log the enqueuing
    msg!(
        "Enqueued messages for execution_id: {}, priority: {}, subroutine: {:?}, count: {}",
        execution_id,
        priority_to_string(&priority_enum),
        message_batch.subroutine_type,
        message_batch.messages.len()
    );
    
    Ok(())
} 