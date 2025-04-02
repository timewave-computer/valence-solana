use anchor_lang::prelude::*;
use anchor_spl::token_2022;
use solana_program::pubkey::Pubkey;

pub mod token_helpers {
    use super::*;

    /// Get the token program ID
    /// 
    /// @return Pubkey The program ID of Token-2022
    pub fn get_token_program_id() -> Pubkey {
        token_2022::ID
    }

    /// Transfer tokens using Token-2022
    /// 
    /// @param ctx The CPI context for the transfer
    /// @param amount The amount of tokens to transfer
    /// @return Result<()> The result of the transfer
    pub fn transfer_tokens<'a, 'b, 'c, 'info>(
        ctx: CpiContext<'a, 'b, 'c, 'info, token_2022::Transfer<'info>>,
        amount: u64,
    ) -> Result<()> {
        token_2022::transfer(ctx, amount)
    }
} 