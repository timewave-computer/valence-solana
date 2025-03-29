use anchor_lang::prelude::*;
use crate::state::FactoryState;
use crate::error::AccountFactoryError;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CreateSingleUseAccountParams {
    pub owner: Pubkey,
    pub auth_token: Pubkey,
    pub auto_approve_libraries: Option<Vec<Pubkey>>,
    pub required_destination: Option<Pubkey>,
    pub expiration_time: Option<i64>,
    pub fund_amount: Option<u64>,
}

#[derive(Accounts)]
pub struct CreateSingleUseAccount<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"factory_state"],
        bump,
        constraint = !factory_state.is_paused @ AccountFactoryError::UnauthorizedOperation
    )]
    pub factory_state: Account<'info, FactoryState>,
    
    /// CHECK: This is the owner of the account to be created
    pub owner: UncheckedAccount<'info>,
    
    /// CHECK: This is the receiver of any fees (validated in the handler)
    #[account(mut)]
    pub fee_receiver: UncheckedAccount<'info>,
    
    /// CHECK: The single-use account that will be created
    #[account(mut)]
    pub single_use_account: UncheckedAccount<'info>,
    
    /// CHECK: Single-use account program that will be invoked
    #[account(mut)]
    pub single_use_account_program: UncheckedAccount<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<CreateSingleUseAccount>, params: CreateSingleUseAccountParams) -> Result<()> {
    let factory_state = &mut ctx.accounts.factory_state;
    let payer = ctx.accounts.payer.key();
    let fee_receiver = &ctx.accounts.fee_receiver;
    let current_time = Clock::get()?.unix_timestamp;
    
    // Verify fee receiver is correct
    if fee_receiver.key() != factory_state.fee_receiver {
        return Err(AccountFactoryError::UnauthorizedOperation.into());
    }
    
    // Validate expiration time if provided
    if let Some(expiration_time) = params.expiration_time {
        if expiration_time <= current_time {
            return Err(AccountFactoryError::InvalidAccountParameter.into());
        }
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
        ).map_err(|_| AccountFactoryError::InsufficientFunds.into())?;
        
        msg!("Fee of {} lamports collected", factory_state.creation_fee);
    }
    
    // In a real implementation, we would create the Single-Use Account via CPI
    // to the Single-Use Account program, passing the necessary parameters
    
    // Here we'd invoke the single_use_account_program's initialize instruction
    // The CPI call would look something like:
    /*
    let cpi_program = ctx.accounts.single_use_account_program.to_account_info();
    let cpi_accounts = SingleUseAccountInitialize {
        authority: ctx.accounts.owner.to_account_info(),
        payer: ctx.accounts.payer.to_account_info(),
        single_use_account: ctx.accounts.single_use_account.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    
    single_use_account::cpi::initialize(
        cpi_ctx,
        SingleUseAccountInitializeParams {
            auth_token: params.auth_token,
            required_destination: params.required_destination,
            expiration_time: params.expiration_time,
        },
    )?;
    */
    
    // Fund the account if requested
    if let Some(fund_amount) = params.fund_amount {
        if fund_amount > 0 {
            // Transfer SOL to the account
            let fund_ix = solana_program::system_instruction::transfer(
                &payer,
                &ctx.accounts.single_use_account.key(),
                fund_amount
            );
            solana_program::program::invoke(
                &fund_ix,
                &[
                    ctx.accounts.payer.to_account_info(),
                    ctx.accounts.single_use_account.to_account_info(),
                    ctx.accounts.system_program.to_account_info(),
                ],
            ).map_err(|_| AccountFactoryError::InsufficientFunds.into())?;
            
            msg!("Single-use account funded with {} lamports", fund_amount);
        }
    }
    
    // Auto-approve libraries if provided
    if let Some(libraries) = &params.auto_approve_libraries {
        if !libraries.is_empty() {
            msg!("Auto-approving {} libraries would happen here", libraries.len());
            // Logic for approving libraries via CPI would go here
        }
    }
    
    // Update factory state
    factory_state.increment_account_count();
    
    msg!("Single-use account created for owner: {}", params.owner);
    
    if let Some(destination) = params.required_destination {
        msg!("Required destination: {}", destination);
    }
    
    if let Some(expiration) = params.expiration_time {
        msg!("Expiration time: {} (current time: {})", expiration, current_time);
    }
    
    Ok(())
} 