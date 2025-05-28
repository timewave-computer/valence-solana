use anchor_lang::prelude::*;
use crate::state::FactoryState;
use crate::error::AccountFactoryError;
use base_account::cpi::{accounts::Initialize, initialize};
use base_account::program::BaseAccount as BaseAccountProgram;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CreateBaseAccountParams {
    pub owner: Pubkey,
    pub max_libraries: u8,
    pub max_token_accounts: u8,
    pub fund_amount: Option<u64>,
}

#[derive(Accounts)]
pub struct CreateBaseAccount<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"factory_state"],
        bump,
        constraint = !factory_state.is_paused @ AccountFactoryError::UnauthorizedOperation
    )]
    pub factory_state: Account<'info, FactoryState>,
    
    /// The owner of the account to be created
    /// This account must sign for the creation
    #[account(mut)]
    pub owner: Signer<'info>,
    
    /// CHECK: This is the receiver of any fees (validated in the handler)
    #[account(mut)]
    pub fee_receiver: UncheckedAccount<'info>,
    
    /// CHECK: The base account that will be created
    #[account(mut)]
    pub base_account: UncheckedAccount<'info>,
    
    /// The Base Account program
    pub base_account_program: Program<'info, BaseAccountProgram>,
    
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<CreateBaseAccount>, params: CreateBaseAccountParams) -> Result<()> {
    let factory_state = &mut ctx.accounts.factory_state;
    let payer = ctx.accounts.payer.key();
    let fee_receiver = &ctx.accounts.fee_receiver;
    
    // Verify fee receiver is correct
    if fee_receiver.key() != factory_state.fee_receiver {
        return Err(anchor_lang::error::Error::from(AccountFactoryError::UnauthorizedOperation));
    }
    
    // Collect fee if set
    if factory_state.creation_fee > 0 {
        let fee_ix = solana_program::system_instruction::transfer(
            &payer,
            &factory_state.fee_receiver,
            factory_state.creation_fee
        );
        solana_program::program::invoke(
            &fee_ix,
            &[
                ctx.accounts.payer.to_account_info(),
                fee_receiver.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        ).map_err(|_| AccountFactoryError::InsufficientFunds)?;
        
        msg!("Fee of {} lamports collected", factory_state.creation_fee);
    }
    
    // Create the Base Account via CPI
    let cpi_program = ctx.accounts.base_account_program.to_account_info();
    let cpi_accounts = Initialize {
        account: ctx.accounts.base_account.to_account_info(),
        signer: ctx.accounts.owner.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    
    initialize(
        cpi_ctx,
        params.max_libraries,
        params.max_token_accounts,
    )?;
    
    // Fund the account if requested
    if let Some(fund_amount) = params.fund_amount {
        if fund_amount > 0 {
            // Transfer SOL to the account
            let fund_ix = solana_program::system_instruction::transfer(
                &payer,
                &ctx.accounts.base_account.key(),
                fund_amount
            );
            solana_program::program::invoke(
                &fund_ix,
                &[
                    ctx.accounts.payer.to_account_info(),
                    ctx.accounts.base_account.to_account_info(),
                    ctx.accounts.system_program.to_account_info(),
                ],
            ).map_err(|_| AccountFactoryError::InsufficientFunds)?;
            
            msg!("Base account funded with {} lamports", fund_amount);
        }
    }
    
    // Update factory state
    factory_state.increment_account_count();
    
    msg!("Base account created for owner: {}", params.owner);
    
    Ok(())
} 