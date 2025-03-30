use anchor_lang::prelude::*;
use crate::state::{FactoryState, AccountTemplate};
use crate::error::AccountFactoryError;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct UpdateTemplateParams {
    pub template_id: String,
    pub description: Option<String>,
    pub is_active: Option<bool>,
    pub auto_fund_sol: Option<bool>,
    pub fund_amount_sol: Option<u64>,
    pub create_token_accounts: Option<bool>,
    pub token_mints_to_add: Option<Vec<Pubkey>>,
    pub token_mints_to_remove: Option<Vec<Pubkey>>,
    pub approve_libraries: Option<bool>,
    pub libraries_to_approve: Option<Vec<Pubkey>>,
    pub libraries_to_revoke: Option<Vec<Pubkey>>,
}

#[derive(Accounts)]
#[instruction(params: UpdateTemplateParams)]
pub struct UpdateTemplate<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        seeds = [b"factory_state"],
        bump,
        constraint = factory_state.authority == authority.key() @ AccountFactoryError::UnauthorizedOperation,
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
        constraint = template.authority == authority.key() @ AccountFactoryError::UnauthorizedOperation
    )]
    pub template: Account<'info, AccountTemplate>,
    
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<UpdateTemplate>, params: UpdateTemplateParams) -> Result<()> {
    let template = &mut ctx.accounts.template;
    let current_time = Clock::get()?.unix_timestamp;
    
    // Update template fields if provided
    if let Some(description) = params.description {
        if description.len() <= 500 {
            template.description = description;
        } else {
            return Err(AccountFactoryError::InvalidTemplateParameters.into());
        }
    }
    
    if let Some(is_active) = params.is_active {
        template.is_active = is_active;
    }
    
    if let Some(auto_fund_sol) = params.auto_fund_sol {
        template.auto_fund_sol = auto_fund_sol;
    }
    
    if let Some(fund_amount_sol) = params.fund_amount_sol {
        template.fund_amount_sol = fund_amount_sol;
    }
    
    if let Some(create_token_accounts) = params.create_token_accounts {
        template.create_token_accounts = create_token_accounts;
    }
    
    // Add new token mints
    if let Some(token_mints_to_add) = params.token_mints_to_add {
        for mint in token_mints_to_add {
            if !template.token_mints.contains(&mint) {
                template.token_mints.push(mint);
            }
        }
    }
    
    // Remove token mints
    if let Some(token_mints_to_remove) = params.token_mints_to_remove {
        template.token_mints.retain(|mint| !token_mints_to_remove.contains(mint));
    }
    
    if let Some(approve_libraries) = params.approve_libraries {
        template.approve_libraries = approve_libraries;
    }
    
    // Add new approved libraries
    if let Some(libraries_to_approve) = params.libraries_to_approve {
        for library in libraries_to_approve {
            if !template.approved_libraries.contains(&library) {
                template.approved_libraries.push(library);
            }
        }
    }
    
    // Remove approved libraries
    if let Some(libraries_to_revoke) = params.libraries_to_revoke {
        template.approved_libraries.retain(|library| !libraries_to_revoke.contains(library));
    }
    
    // Increment version and update timestamp
    template.version += 1;
    template.last_update = current_time;
    
    msg!("Template updated: {}", template.template_id);
    msg!("New version: {}", template.version);
    
    Ok(())
} 