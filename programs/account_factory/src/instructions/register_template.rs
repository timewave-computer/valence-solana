use anchor_lang::prelude::*;
use crate::state::{FactoryState, AccountTemplate};
use crate::error::AccountFactoryError;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct RegisterTemplateParams {
    pub template_id: String,
    pub account_type: u8,
    pub description: String,
    pub auto_fund_sol: bool,
    pub fund_amount_sol: u64,
    pub create_token_accounts: bool,
    pub token_mints: Vec<Pubkey>,
    pub approve_libraries: bool,
    pub approved_libraries: Vec<Pubkey>,
}

#[derive(Accounts)]
#[instruction(params: RegisterTemplateParams)]
pub struct RegisterTemplate<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"factory_state"],
        bump,
        constraint = factory_state.authority == authority.key() @ AccountFactoryError::UnauthorizedOperation,
        constraint = !factory_state.is_paused @ AccountFactoryError::UnauthorizedOperation
    )]
    pub factory_state: Account<'info, FactoryState>,
    
    #[account(
        init,
        payer = authority,
        space = AccountTemplate::size(
            params.template_id.len(),
            params.token_mints.len(),
            params.approved_libraries.len(),
            params.description.len()
        ),
        seeds = [
            b"account_template",
            params.template_id.as_bytes()
        ],
        bump
    )]
    pub template: Account<'info, AccountTemplate>,
    
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<RegisterTemplate>, params: RegisterTemplateParams) -> Result<()> {
    let factory_state = &mut ctx.accounts.factory_state;
    let template = &mut ctx.accounts.template;
    let authority = ctx.accounts.authority.key();
    let current_time = Clock::get()?.unix_timestamp;
    
    // Validate parameters
    if params.template_id.is_empty() || params.template_id.len() > 100 {
        return Err(AccountFactoryError::InvalidTemplateParameters.into());
    }
    
    if params.description.len() > 500 {
        return Err(AccountFactoryError::InvalidTemplateParameters.into());
    }
    
    if params.account_type > 1 {
        return Err(AccountFactoryError::InvalidAccountType.into());
    }
    
    // Initialize template
    template.template_id = params.template_id.clone();
    template.authority = authority;
    template.account_type = params.account_type;
    template.version = 1; // Initial version
    template.is_active = true;
    template.auto_fund_sol = params.auto_fund_sol;
    template.fund_amount_sol = params.fund_amount_sol;
    template.create_token_accounts = params.create_token_accounts;
    template.token_mints = params.token_mints;
    template.approve_libraries = params.approve_libraries;
    template.approved_libraries = params.approved_libraries;
    template.description = params.description;
    template.last_update = current_time;
    template.usage_count = 0;
    template.reserved = [0; 64];
    
    // Update factory state
    factory_state.increment_template_count();
    
    msg!("Template registered: {}", template.template_id);
    msg!("Account type: {}", template.account_type);
    
    Ok(())
} 