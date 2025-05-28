use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::ProcessorError;

pub fn handler(
    ctx: Context<crate::state::EnqueueMessages>,
    execution_id: u64,
    priority: u8,
    subroutine_type: u8,
    messages: Vec<ProcessorMessage>,
) -> Result<()> {
    let processor_state = &mut ctx.accounts.processor_state;
    
    // Validate inputs
    if messages.is_empty() {
        return Err(ProcessorError::InvalidMessageFormat.into());
    }
    
    if processor_state.is_paused {
        return Err(ProcessorError::ProcessorPaused.into());
    }
    
    // Convert priority and subroutine type
    let priority_level = match priority {
        0 => Priority::Low,
        1 => Priority::Medium,
        2 => Priority::High,
        _ => return Err(ProcessorError::InvalidPriorityLevel.into()),
    };
    
    let sub_type = match subroutine_type {
        0 => SubroutineType::Atomic,
        1 => SubroutineType::NonAtomic,
        _ => return Err(ProcessorError::InvalidSubroutineType.into()),
    };
    
    // Create message batch
    let message_batch = MessageBatch {
        execution_id,
        messages,
        subroutine_type: sub_type,
        expiration_time: Some(Clock::get()?.unix_timestamp + 3600), // 1 hour expiry
        priority: priority_level.clone(),
        caller: ctx.accounts.caller.key(),
        callback_address: ctx.accounts.callback_address.key(), // Use callback address
        created_at: Clock::get()?.unix_timestamp,
        bump: 255, // This would be derived properly in real implementation
    };
    
    // Select appropriate queue based on priority
    let queue = match message_batch.priority {
        Priority::High => &mut processor_state.high_priority_queue,
        Priority::Medium => &mut processor_state.medium_priority_queue,
        Priority::Low => &mut processor_state.low_priority_queue,
    };
    
    // Check queue capacity
    if queue.count >= queue.capacity {
        return Err(ProcessorError::QueueFull.into());
    }
    
    // Add to queue (in real implementation, this would store the batch)
    queue.count += 1;
    processor_state.last_execution_time = Clock::get()?.unix_timestamp;
    
    msg!("Enqueued {} messages with execution ID {} in {:?} priority queue", 
         message_batch.messages.len(), execution_id, message_batch.priority);
    
    Ok(())
}

 