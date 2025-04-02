use anchor_lang::prelude::*;
use crate::state::AccountState;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitializeParams {
    pub auth_token: Pubkey,
}

pub fn initialize(
    ctx: CpiContext<'_, '_, '_, '_, Initialize>,
    params: InitializeParams,
) -> Result<()> {
    let ix = anchor_lang::solana_program::instruction::Instruction {
        program_id: ctx.program.key(),
        accounts: anchor_lang::ToAccountMetas::to_account_metas(
            &ctx.accounts,
            None,
        ),
        data: anchor_lang::InstructionData::data(&crate::instruction::BaseAccountInstruction::Initialize(
            params,
        )),
    };
    anchor_lang::solana_program::program::invoke_signed(
        &ix,
        &anchor_lang::ToAccountInfos::to_account_infos(&ctx),
        ctx.signer_seeds,
    )
    .map_err(Into::into)
}

pub fn approve_library(
    ctx: CpiContext<'_, '_, '_, '_, ApproveLibrary>,
    library: Pubkey,
) -> Result<()> {
    let ix = anchor_lang::solana_program::instruction::Instruction {
        program_id: ctx.program.key(),
        accounts: anchor_lang::ToAccountMetas::to_account_metas(
            &ctx.accounts,
            None,
        ),
        data: anchor_lang::InstructionData::data(&crate::instruction::BaseAccountInstruction::ApproveLibrary(
            library,
        )),
    };
    anchor_lang::solana_program::program::invoke_signed(
        &ix,
        &anchor_lang::ToAccountInfos::to_account_infos(&ctx),
        ctx.signer_seeds,
    )
    .map_err(Into::into)
}

pub mod accounts {
    use super::*;

    #[derive(Accounts)]
    pub struct Initialize<'info> {
        #[account(mut)]
        pub authority: Signer<'info>,
        #[account(mut)]
        /// CHECK: Validated in the handler logic
        pub base_account: UncheckedAccount<'info>,
        pub system_program: Program<'info, System>,
    }

    #[derive(Accounts)]
    pub struct ApproveLibrary<'info> {
        #[account(mut)]
        pub authority: Signer<'info>,
        #[account(mut)]
        pub base_account: Account<'info, AccountState>,
    }
} 