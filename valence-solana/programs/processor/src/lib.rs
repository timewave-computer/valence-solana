use anchor_lang::prelude::*;

// Program ID declaration
declare_id!("ProcessorProgramxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");

pub mod state;
pub mod instructions;
pub mod error;
pub mod queue;

use state::*;
use instructions::*;
use error::*;
use queue::*;

#[program]
pub mod processor {
    use super::*;

    /// Initialize the Processor Program with authorization program
    pub fn initialize(
        ctx: Context<Initialize>,
        authorization_program_id: Pubkey,
    ) -> Result<()> {
        instructions::initialize::handler(ctx, authorization_program_id)
    }

    /// Enqueue messages for processing
    pub fn enqueue_messages(
        ctx: Context<EnqueueMessages>,
        execution_id: u64,
        priority: u8,
        subroutine_type: u8,
        messages: Vec<ProcessorMessage>,
    ) -> Result<()> {
        instructions::enqueue_messages::handler(ctx, execution_id, priority, subroutine_type, messages)
    }

    /// Process the next available batch of messages
    pub fn process_tick(ctx: Context<ProcessTick>) -> Result<()> {
        instructions::process_tick::handler(ctx)
    }

    /// Send a callback to the authorization program
    pub fn send_callback(
        ctx: Context<SendCallback>,
        execution_id: u64,
        result: ExecutionResult,
        executed_count: u32,
        error_data: Option<Vec<u8>>,
    ) -> Result<()> {
        instructions::send_callback::handler(ctx, execution_id, result, executed_count, error_data)
    }

    /// Pause the processor
    pub fn pause_processor(ctx: Context<PauseProcessor>) -> Result<()> {
        instructions::pause_processor::handler(ctx)
    }

    /// Resume the processor
    pub fn resume_processor(ctx: Context<ResumeProcessor>) -> Result<()> {
        instructions::resume_processor::handler(ctx)
    }
}

/// The source file structure will be:
/// 
/// lib.rs - Main program entry point with instruction routing
/// state.rs - Account structures and data types
/// error.rs - Error handling for the program
/// queue.rs - Implementation of priority queues
/// instructions/ - Individual instruction handlers
///    mod.rs - Module exports
///    initialize.rs - Handler for initialize instruction
///    enqueue_messages.rs - Handler for enqueuing messages
///    process_tick.rs - Handler for processing messages
///    send_callback.rs - Handler for sending callbacks
///    pause_processor.rs - Handler for pausing the processor
///    resume_processor.rs - Handler for resuming the processor
///
/// 