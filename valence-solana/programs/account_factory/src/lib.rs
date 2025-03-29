use anchor_lang::prelude::*;

declare_id!("AccFctyYYfhQTqQyP7NaSxna4HLnQW5XjJFKhQdjitA");

pub mod error;
pub mod state;
pub mod instructions;

use instructions::*;

#[program]
pub mod account_factory {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, params: InitializeParams) -> Result<()> {
        instructions::initialize::handler(ctx, params)
    }

    pub fn create_base_account(ctx: Context<CreateBaseAccount>, params: CreateBaseAccountParams) -> Result<()> {
        instructions::create_base_account::handler(ctx, params)
    }

    pub fn create_storage_account(ctx: Context<CreateStorageAccount>, params: CreateStorageAccountParams) -> Result<()> {
        instructions::create_storage_account::handler(ctx, params)
    }

    pub fn create_single_use_account(ctx: Context<CreateSingleUseAccount>, params: CreateSingleUseAccountParams) -> Result<()> {
        instructions::create_single_use_account::handler(ctx, params)
    }

    pub fn register_template(ctx: Context<RegisterTemplate>, params: RegisterTemplateParams) -> Result<()> {
        instructions::register_template::handler(ctx, params)
    }

    pub fn update_template(ctx: Context<UpdateTemplate>, params: UpdateTemplateParams) -> Result<()> {
        instructions::update_template::handler(ctx, params)
    }

    pub fn create_from_template(ctx: Context<CreateFromTemplate>, params: CreateFromTemplateParams) -> Result<()> {
        instructions::create_from_template::handler(ctx, params)
    }

    pub fn batch_create_accounts(ctx: Context<BatchCreateAccounts>, params: BatchCreateAccountsParams) -> Result<()> {
        instructions::batch_create_accounts::handler(ctx, params)
    }
} 