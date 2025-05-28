use anchor_lang::prelude::*;
use crate::state::*;

pub fn handler(_ctx: Context<ProcessTick>) -> Result<()> {
    msg!("Process tick placeholder");
    Ok(())
} 