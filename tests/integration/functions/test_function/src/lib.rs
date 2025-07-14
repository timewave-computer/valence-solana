use anchor_lang::prelude::*;

declare_id!("8r2SeUcUmdzXHuvsNDsNxCPLkn8w6Jz9z1wtLk3ChzNR");

pub const FUNCTION_STATE_SEED: &[u8] = b"function_state";

#[program]
pub mod test_function {
    use super::*;

    /// Initialize function state for a user
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let state = &mut ctx.accounts.function_state;
        state.owner = ctx.accounts.owner.key();
        state.counter = 0;
        state.last_value = 0;
        state.data = Vec::new();
        
        msg!("Initialized function state for {}", state.owner);
        Ok(())
    }

    /// Test function that processes input data
    /// This is a minimal implementation that demonstrates:
    /// 1. The shard program has already validated session and capabilities
    /// 2. Processing input data according to function logic
    /// 3. Updating state when state account is provided
    pub fn process(ctx: Context<Process>, input_data: Vec<u8>) -> Result<()> {
        msg!("Test function processing {} bytes of data", input_data.len());
        
        // The session has already been validated by the shard program
        // Functions trust that the shard has done proper capability checking
        msg!("Processing with session account: {}", ctx.accounts.session.key());
        msg!("Owner: {}", ctx.accounts.owner.key());
        
        // Process the input data according to function logic
        if input_data.is_empty() {
            msg!("No input data provided");
            return Ok(());
        }
        
        // Check if we have a state account to work with
        let has_state = ctx.remaining_accounts.len() > 0;
        
        match input_data[0] {
            0x01 => {
                // ECHO - The shard program has already verified READ capability
                msg!("Command: ECHO");
                
                if input_data.len() > 1 {
                    let echo_data = &input_data[1..];
                    msg!("Echo data: {:?}", echo_data);
                    
                    if has_state {
                        // In a real implementation, we would update the state account
                        msg!("Would store {} bytes in state account", echo_data.len());
                    } else {
                        msg!("No state account provided, simulating echo");
                    }
                }
            },
            0x02 => {
                // COMPUTE - The shard program has already verified EXECUTE capability
                msg!("Command: COMPUTE");
                
                if input_data.len() >= 9 {
                    let a = u32::from_le_bytes([input_data[1], input_data[2], input_data[3], input_data[4]]);
                    let b = u32::from_le_bytes([input_data[5], input_data[6], input_data[7], input_data[8]]);
                    let result = a.checked_add(b)
                        .ok_or(TestFunctionError::ArithmeticOverflow)?;
                    
                    msg!("Computed {} + {} = {}", a, b, result);
                    
                    if has_state {
                        msg!("Would update state with result: {}", result);
                    } else {
                        msg!("No state account provided, result computed but not stored");
                    }
                }
            },
            0x03 => {
                // VERIFY - The shard program has already verified READ capability
                msg!("Command: VERIFY");
                
                msg!("Session verification:");
                msg!("  Session account: {}", ctx.accounts.session.key());
                msg!("  Owner: {} ✓", ctx.accounts.owner.key());
                msg!("  Capability check: Already validated by shard");
                
                if has_state {
                    msg!("  State account available: Yes");
                } else {
                    msg!("  State account available: No");
                }
            },
            0x04 => {
                // STORE - The shard program has already verified WRITE capability
                msg!("Command: STORE");
                
                if input_data.len() > 1 {
                    let store_data = &input_data[1..];
                    
                    if has_state {
                        msg!("Would store {} bytes in state account", store_data.len());
                        // In a real implementation, we would write to the state account
                    } else {
                        msg!("No state account provided, cannot store data");
                        return Err(TestFunctionError::NoStateAccount.into());
                    }
                }
            },
            0x05 => {
                // TRANSFER (simulation) - The shard program has already verified TRANSFER capability
                msg!("Command: TRANSFER");
                
                if input_data.len() >= 9 {
                    let amount = u32::from_le_bytes([input_data[1], input_data[2], input_data[3], input_data[4]]);
                    let recipient_bytes = &input_data[5..9];
                    
                    msg!("Transfer simulation: {} units to {:?}", amount, recipient_bytes);
                    msg!("Note: In a real implementation, this would transfer tokens or SOL");
                    msg!("Transfer processed (simulation only)");
                }
            },
            _ => {
                msg!("Unknown command: {:#x}", input_data[0]);
                return Err(TestFunctionError::UnknownCommand.into());
            }
        }
        
        msg!("Test function completed successfully");
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = owner,
        space = FunctionState::SIZE,
        seeds = [FUNCTION_STATE_SEED, owner.key().as_ref()],
        bump
    )]
    pub function_state: Account<'info, FunctionState>,
    
    #[account(mut)]
    pub owner: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Process<'info> {
    /// The session account that authorized this function call
    /// CHECK: Validated by the shard program before CPI
    pub session: AccountInfo<'info>,
    
    /// The owner of the session
    pub owner: Signer<'info>,
}

#[account]
pub struct FunctionState {
    pub owner: Pubkey,
    pub counter: u32,
    pub last_value: u32,
    pub data: Vec<u8>,
}

impl FunctionState {
    pub const SIZE: usize = 8 + // discriminator
        32 + // owner
        4 + // counter
        4 + // last_value
        4 + 256; // data vec (4 bytes length + up to 256 bytes data)
}

#[error_code]
pub enum TestFunctionError {
    #[msg("Unauthorized owner")]
    UnauthorizedOwner,
    #[msg("Unknown command")]
    UnknownCommand,
    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,
    #[msg("No state account provided")]
    NoStateAccount,
}