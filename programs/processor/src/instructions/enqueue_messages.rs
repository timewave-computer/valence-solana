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
    msg!("Enqueue messages placeholder");
    Ok(())
} 