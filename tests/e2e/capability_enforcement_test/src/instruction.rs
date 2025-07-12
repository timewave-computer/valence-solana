//! Instruction definitions for the template project

use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct Initialize {}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct Echo {
    pub message: String,
}

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct Transfer {
    pub amount: u64,
    pub recipient: Pubkey,
}