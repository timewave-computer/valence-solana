use anchor_lang::prelude::*;
use crate::instructions::initialize::{Initialize, InitializeParams};
use anchor_lang::solana_program::{
    instruction::{AccountMeta, Instruction},
    program::invoke_signed,
};

// CPI function for initializing a base account
pub fn initialize<'a, 'b, 'c, 'info>(
    ctx: CpiContext<'a, 'b, 'c, 'info, Initialize<'info>>,
    params: InitializeParams,
) -> Result<()> {
    let ix = Instruction {
        program_id: ctx.program.key(),
        accounts: Initialize::to_account_metas(&ctx.accounts, None),
        data: crate::instruction::BaseAccountInstruction::Initialize(params)
            .data(),
    };

    invoke_signed(
        &ix,
        &[
            ctx.accounts.authority.to_account_info(),
            ctx.accounts.base_account.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
        ctx.signer_seeds,
    )
    .map_err(Into::into)
}

#[derive(Clone)]
pub struct BaseAccountProgram;

impl anchor_lang::Id for BaseAccountProgram {
    fn id() -> Pubkey {
        crate::ID
    }
} 