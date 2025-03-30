use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke;
use anchor_lang::solana_program::instruction::Instruction;
use crate::state::*;
use crate::error::ProcessorError;

pub fn handler(
    ctx: Context<SendCallback>,
    execution_id: u64,
    result: ExecutionResult,
    executed_count: u32,
    error_data: Option<Vec<u8>>
) -> Result<()> {
    msg!("Send callback placeholder");
    Ok(())
} 