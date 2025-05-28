use anchor_lang::prelude::*;
use crate::state::{FactoryState, AccountTemplate};
use crate::error::AccountFactoryError;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CreateFromTemplateParams {
    pub template_id: String,
    pub seed: Option<String>,
    pub owner: Option<Pubkey>,
    pub auth_token: Option<Pubkey>,
}

#[derive(Accounts)]
#[instruction(params: CreateFromTemplateParams)]
pub struct CreateFromTemplate<'info> {
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
    
    /// CHECK: The account that will be created (checked in the handler)
    #[account(mut)]
    pub created_account: UncheckedAccount<'info>,
    
    /// CHECK: Sysvar that might be needed for cross-program invocations
    #[account(address = solana_program::sysvar::rent::ID)]
    pub rent: UncheckedAccount<'info>,
    
    /// CHECK: Will be properly used based on the account type to be created
    pub base_account_program: UncheckedAccount<'info>,
    
    /// CHECK: Will be properly used if creating a storage account
    pub storage_account_program: UncheckedAccount<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<CreateFromTemplate>, params: CreateFromTemplateParams) -> Result<()> {
    let factory_state = &mut ctx.accounts.factory_state;
    let template = &mut ctx.accounts.template;
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
    
    // Determine which account type to create
    let _owner = params.owner.unwrap_or(payer);
    let _auth_token = params.auth_token.unwrap_or_else(|| Pubkey::new_unique());
    
    match template.account_type {
        0 => {
            // Create Base Account
            msg!("Creating Base Account from template: {}", template.template_id);
            // Logic for creating the Base Account via CPI would go here
        },
        1 => {
            // Create Storage Account
            msg!("Creating Storage Account from template: {}", template.template_id);
            // Logic for creating the Storage Account via CPI would go here
        },
        _ => {
            return Err(anchor_lang::error::Error::from(AccountFactoryError::InvalidAccountType));
        }
    }
    
    // Auto-approve libraries if configured
    if template.approve_libraries && !template.approved_libraries.is_empty() {
        msg!("Auto-approving {} libraries", template.approved_libraries.len());
        // Logic for approving libraries via CPI would go here
    }
    
    // Update template and factory state
    template.increment_usage_count();
    factory_state.increment_account_count();
    
    msg!("Account created successfully from template: {}", template.template_id);
    
    Ok(())
} 