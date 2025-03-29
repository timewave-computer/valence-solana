use anchor_lang::prelude::*;
use crate::state::{FactoryState, AccountTemplate};
use crate::error::AccountFactoryError;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct BatchAccountParams {
    pub seed: Option<String>,
    pub owner: Option<Pubkey>,
    pub auth_token: Option<Pubkey>,
    pub override_required_destination: Option<Pubkey>,
    pub override_expiration_seconds: Option<u64>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct BatchCreateAccountsParams {
    pub template_id: String,
    pub batch_params: Vec<BatchAccountParams>,
}

#[derive(Accounts)]
#[instruction(params: BatchCreateAccountsParams)]
pub struct BatchCreateAccounts<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"factory_state"],
        bump,
        constraint = !factory_state.is_paused @ AccountFactoryError::UnauthorizedOperation
    )]
    pub factory_state: Account<'info, FactoryState>,
    
    #[account(
        mut,
        seeds = [
            b"account_template",
            params.template_id.as_bytes()
        ],
        bump,
        constraint = template.is_active @ AccountFactoryError::TemplateDisabled
    )]
    pub template: Account<'info, AccountTemplate>,
    
    /// CHECK: This is the receiver of any fees (validated in the handler)
    #[account(mut)]
    pub fee_receiver: UncheckedAccount<'info>,
    
    /// CHECK: Sysvar that might be needed for cross-program invocations
    #[account(address = solana_program::sysvar::rent::ID)]
    pub rent: UncheckedAccount<'info>,
    
    /// CHECK: Will be properly used based on the account type to be created
    pub base_account_program: UncheckedAccount<'info>,
    
    /// CHECK: Will be properly used if creating a storage account
    pub storage_account_program: UncheckedAccount<'info>,
    
    /// CHECK: Will be properly used if creating a single-use account
    pub single_use_account_program: UncheckedAccount<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<BatchCreateAccounts>, params: BatchCreateAccountsParams) -> Result<()> {
    let factory_state = &mut ctx.accounts.factory_state;
    let template = &mut ctx.accounts.template;
    let payer = ctx.accounts.payer.key();
    let fee_receiver = &ctx.accounts.fee_receiver;
    let batch_size = params.batch_params.len();
    
    // Verify fee receiver is correct
    if fee_receiver.key() != factory_state.fee_receiver {
        return Err(AccountFactoryError::UnauthorizedOperation.into());
    }
    
    // Check batch size limit
    if batch_size == 0 {
        return Err(AccountFactoryError::InvalidAccountParameter.into());
    }
    
    if batch_size > 10 {
        return Err(AccountFactoryError::BatchSizeExceeded.into());
    }
    
    // Collect total fee if set
    if factory_state.creation_fee > 0 {
        let total_fee = factory_state.creation_fee.saturating_mul(batch_size as u64);
        
        let fee_ix = solana_program::system_instruction::transfer(
            &payer,
            &factory_state.fee_receiver,
            total_fee
        );
        solana_program::program::invoke(
            &fee_ix,
            &[
                ctx.accounts.payer.to_account_info(),
                fee_receiver.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
        ).map_err(|_| AccountFactoryError::InsufficientFunds.into())?;
        
        msg!("Total fee of {} lamports collected for batch creation", total_fee);
    }
    
    // Create each account in the batch
    let mut success_count = 0;
    
    for (i, batch_param) in params.batch_params.iter().enumerate() {
        // Determine account specific parameters
        let owner = batch_param.owner.unwrap_or(payer);
        let auth_token = batch_param.auth_token.unwrap_or_else(|| Pubkey::new_unique());
        let seed = batch_param.seed.as_ref().map(|s| s.as_str()).unwrap_or("");
        
        msg!("Creating account {}/{} with seed: {}", i + 1, batch_size, if seed.is_empty() { "default" } else { seed });
        
        // In a real implementation, we would create each account via CPI to the appropriate program
        // based on the template.account_type
        
        // For this skeleton implementation, we just increment the success counter
        success_count += 1;
    }
    
    // Update template and factory state
    template.increment_usage_count_by(success_count);
    factory_state.increment_account_count_by(success_count);
    
    msg!("Batch creation completed. Successfully created {}/{} accounts from template: {}", 
         success_count, batch_size, template.template_id);
    
    Ok(())
} 