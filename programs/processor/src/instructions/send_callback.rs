use anchor_lang::prelude::*;
use crate::state::*;

pub fn handler(
    _ctx: Context<SendCallback>,
    _execution_id: u64,
    _result: ExecutionResult,
    _executed_count: u32,
    _error_data: Option<Vec<u8>>
) -> Result<()> {
    msg!("Send callback placeholder");
    Ok(())
} 