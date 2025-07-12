//! Test functions that demonstrate proper capability-based access

use anchor_lang::prelude::*;

declare_id!("Func1111111111111111111111111111111111111111");

#[program]
pub mod test_functions {
    use super::*;

    /// Function that reads external state - requires READ capability
    pub fn read_external_state(ctx: Context<ReadExternal>) -> Result<()> {
        msg!("Function: Reading external state with READ capability");
        
        // Build CPI to external program
        let cpi_accounts = mock_external_program::cpi::accounts::ReadState {
            external_state: ctx.accounts.external_state.to_account_info(),
        };
        
        let cpi_ctx = CpiContext::new(
            ctx.accounts.external_program.to_account_info(),
            cpi_accounts,
        );
        
        // Execute CPI
        mock_external_program::cpi::read_state(cpi_ctx)?;
        
        // Get return data
        let (_program_id, data) = anchor_lang::solana_program::program::get_return_data()
            .ok_or(ProgramError::InvalidAccountData)?;
            
        msg!("Read external data: {:?}", data);
        
        // Return the data
        anchor_lang::solana_program::program::set_return_data(&data);
        Ok(())
    }

    /// Function that writes external state - requires WRITE capability
    pub fn write_external_state(
        ctx: Context<WriteExternal>, 
        new_data: u64
    ) -> Result<()> {
        msg!("Function: Writing external state with WRITE capability");
        
        // Build CPI to external program
        let cpi_accounts = mock_external_program::cpi::accounts::UpdateState {
            authority: ctx.accounts.authority.to_account_info(),
            external_state: ctx.accounts.external_state.to_account_info(),
        };
        
        let cpi_ctx = CpiContext::new(
            ctx.accounts.external_program.to_account_info(),
            cpi_accounts,
        );
        
        // Execute CPI
        mock_external_program::cpi::update_state(cpi_ctx, new_data)?;
        
        msg!("External state updated successfully");
        Ok(())
    }

    /// Function that transfers tokens - requires TRANSFER capability
    pub fn transfer_external_tokens(
        ctx: Context<TransferExternal>,
        amount: u64,
    ) -> Result<()> {
        msg!("Function: Transferring tokens with TRANSFER capability");
        
        // Build CPI to external program
        let cpi_accounts = mock_external_program::cpi::accounts::TransferTokens {
            authority: ctx.accounts.authority.to_account_info(),
            from: ctx.accounts.from.to_account_info(),
            to: ctx.accounts.to.to_account_info(),
        };
        
        let cpi_ctx = CpiContext::new(
            ctx.accounts.external_program.to_account_info(),
            cpi_accounts,
        );
        
        // Execute CPI
        mock_external_program::cpi::transfer_tokens(cpi_ctx, amount)?;
        
        msg!("Token transfer completed");
        Ok(())
    }

    /// Function that requires multiple capabilities - READ + WRITE
    pub fn read_modify_write(
        ctx: Context<ReadModifyWrite>,
        increment: u64,
    ) -> Result<()> {
        msg!("Function: Read-modify-write with READ + WRITE capabilities");
        
        // First read the current state
        let read_accounts = mock_external_program::cpi::accounts::ReadState {
            external_state: ctx.accounts.external_state.to_account_info(),
        };
        
        let read_ctx = CpiContext::new(
            ctx.accounts.external_program.to_account_info(),
            read_accounts,
        );
        
        mock_external_program::cpi::read_state(read_ctx)?;
        
        // Get current value
        let (_program_id, data) = anchor_lang::solana_program::program::get_return_data()
            .ok_or(ProgramError::InvalidAccountData)?;
        
        let current_value = u64::from_le_bytes(data[..8].try_into().unwrap());
        let new_value = current_value + increment;
        
        // Write back the modified value
        let write_accounts = mock_external_program::cpi::accounts::UpdateState {
            authority: ctx.accounts.authority.to_account_info(),
            external_state: ctx.accounts.external_state.to_account_info(),
        };
        
        let write_ctx = CpiContext::new(
            ctx.accounts.external_program.to_account_info(),
            write_accounts,
        );
        
        mock_external_program::cpi::update_state(write_ctx, new_value)?;
        
        msg!("Read-modify-write completed: {} -> {}", current_value, new_value);
        Ok(())
    }

    /// Malicious function that tries direct CPI without going through proper channels
    /// This should be caught by capability checks
    pub fn malicious_direct_cpi(ctx: Context<MaliciousCpi>) -> Result<()> {
        msg!("Malicious: Attempting direct CPI without capabilities");
        
        // Try to directly invoke external program
        let ix = anchor_lang::solana_program::instruction::Instruction {
            program_id: ctx.accounts.external_program.key(),
            accounts: vec![
                anchor_lang::solana_program::instruction::AccountMeta::new(
                    ctx.accounts.external_state.key(),
                    false,
                ),
            ],
            data: vec![0, 1, 2, 3], // Some arbitrary instruction data
        };
        
        anchor_lang::solana_program::program::invoke(
            &ix,
            &[
                ctx.accounts.external_state.to_account_info(),
                ctx.accounts.external_program.to_account_info(),
            ],
        )?;
        
        msg!("Direct CPI completed (this should not happen!)");
        Ok(())
    }
}

#[derive(Accounts)]
pub struct ReadExternal<'info> {
    /// CHECK: External state account
    pub external_state: AccountInfo<'info>,
    /// CHECK: External program
    pub external_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct WriteExternal<'info> {
    pub authority: Signer<'info>,
    /// CHECK: External state account
    pub external_state: AccountInfo<'info>,
    /// CHECK: External program
    pub external_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct TransferExternal<'info> {
    pub authority: Signer<'info>,
    /// CHECK: From account
    pub from: AccountInfo<'info>,
    /// CHECK: To account
    pub to: AccountInfo<'info>,
    /// CHECK: External program
    pub external_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct ReadModifyWrite<'info> {
    pub authority: Signer<'info>,
    /// CHECK: External state account
    pub external_state: AccountInfo<'info>,
    /// CHECK: External program
    pub external_program: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct MaliciousCpi<'info> {
    /// CHECK: External state account
    pub external_state: AccountInfo<'info>,
    /// CHECK: External program
    pub external_program: AccountInfo<'info>,
}

// Re-export mock external program for CPI
use crate::mock_external_program;