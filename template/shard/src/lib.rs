use anchor_lang::prelude::*;

declare_id!("11111111111111111111111111111111");

#[program]
pub mod template_shard {
    use super::*;

    /// Initialize the shard
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let shard = &mut ctx.accounts.shard;
        shard.authority = ctx.accounts.authority.key();
        shard.function_count = 0;
        
        msg!("Shard initialized with authority: {}", shard.authority);
        Ok(())
    }

    /// Register a function in the shard
    pub fn register_function(
        ctx: Context<RegisterFunction>,
        function_name: String,
        function_hash: [u8; 32],
    ) -> Result<()> {
        let shard = &mut ctx.accounts.shard;
        
        require!(
            ctx.accounts.authority.key() == shard.authority,
            ShardError::UnauthorizedAuthority
        );
        
        // In a real implementation, you would store this in a separate account
        shard.function_count += 1;
        
        msg!(
            "Function '{}' registered with hash: {:?}", 
            function_name, 
            function_hash
        );
        Ok(())
    }

    /// Execute a function through the shard
    pub fn execute_function(
        ctx: Context<ExecuteFunction>,
        function_hash: [u8; 32],
        input_data: Vec<u8>,
    ) -> Result<()> {
        msg!("Executing function with hash: {:?}", function_hash);
        msg!("Input data length: {}", input_data.len());
        
        // In a real implementation, you would:
        // 1. Verify the function is registered
        // 2. Check permissions/capabilities
        // 3. CPI to the actual function program
        // 4. Handle the response
        
        msg!("Function execution completed");
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = authority,
        space = Shard::SIZE,
        seeds = [b"shard"],
        bump
    )]
    pub shard: Account<'info, Shard>,
    
    #[account(mut)]
    pub authority: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct RegisterFunction<'info> {
    #[account(
        mut,
        seeds = [b"shard"],
        bump
    )]
    pub shard: Account<'info, Shard>,
    
    pub authority: Signer<'info>,
}

#[derive(Accounts)]
pub struct ExecuteFunction<'info> {
    #[account(
        seeds = [b"shard"],
        bump
    )]
    pub shard: Account<'info, Shard>,
    
    pub user: Signer<'info>,
}

#[account]
pub struct Shard {
    pub authority: Pubkey,
    pub function_count: u32,
}

impl Shard {
    pub const SIZE: usize = 8 + // discriminator
        32 + // authority
        4;   // function_count
}

#[error_code]
pub enum ShardError {
    #[msg("Unauthorized authority")]
    UnauthorizedAuthority,
    #[msg("Function not found")]
    FunctionNotFound,
    #[msg("Invalid input data")]
    InvalidInputData,
}