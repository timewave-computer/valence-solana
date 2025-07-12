//! Mock external program for testing shard encapsulation

use anchor_lang::prelude::*;

declare_id!("ExtP1111111111111111111111111111111111111111");

#[program]
pub mod mock_external_program {
    use super::*;

    /// Initialize external state
    pub fn initialize(ctx: Context<Initialize>, data: u64) -> Result<()> {
        let external_state = &mut ctx.accounts.external_state;
        external_state.authority = ctx.accounts.authority.key();
        external_state.data = data;
        external_state.counter = 0;
        msg!("External state initialized with data: {}", data);
        Ok(())
    }

    /// Update external state (requires permission)
    pub fn update_state(ctx: Context<UpdateState>, new_data: u64) -> Result<()> {
        let external_state = &mut ctx.accounts.external_state;
        external_state.data = new_data;
        external_state.counter += 1;
        msg!("External state updated to: {}", new_data);
        Ok(())
    }

    /// Read external state (public)
    pub fn read_state(ctx: Context<ReadState>) -> Result<()> {
        let external_state = &ctx.accounts.external_state;
        msg!("External state - data: {}, counter: {}", 
            external_state.data, 
            external_state.counter
        );
        
        // Return the data
        anchor_lang::solana_program::program::set_return_data(
            &external_state.data.to_le_bytes()
        );
        Ok(())
    }

    /// Transfer tokens (requires permission)
    pub fn transfer_tokens(
        ctx: Context<TransferTokens>, 
        amount: u64
    ) -> Result<()> {
        msg!("Transferring {} tokens from {} to {}", 
            amount,
            ctx.accounts.from.key(),
            ctx.accounts.to.key()
        );
        
        // In a real implementation, this would use SPL token transfer
        // For testing, we just log and return success
        anchor_lang::solana_program::program::set_return_data(&[1u8]);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 8 + 8, // discriminator + authority + data + counter
    )]
    pub external_state: Account<'info, ExternalState>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct UpdateState<'info> {
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        constraint = external_state.authority == authority.key() @ ErrorCode::Unauthorized
    )]
    pub external_state: Account<'info, ExternalState>,
}

#[derive(Accounts)]
pub struct ReadState<'info> {
    pub external_state: Account<'info, ExternalState>,
}

#[derive(Accounts)]
pub struct TransferTokens<'info> {
    pub authority: Signer<'info>,
    /// CHECK: Mock token account
    pub from: AccountInfo<'info>,
    /// CHECK: Mock token account
    pub to: AccountInfo<'info>,
}

#[account]
pub struct ExternalState {
    pub authority: Pubkey,
    pub data: u64,
    pub counter: u64,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Unauthorized access")]
    Unauthorized,
}