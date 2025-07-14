use anchor_lang::prelude::*;

declare_id!("22222222222222222222222222222222");

#[program]
pub mod math_ops_function {
    use super::*;

    /// Process math operations
    pub fn process(ctx: Context<Process>, input_data: Vec<u8>) -> Result<()> {
        msg!("Math operations function called");
        
        // Deserialize input
        let input = MathInput::try_from_slice(&input_data)?;
        
        // Perform operation
        let result = match input.operation {
            MathOperation::Add => {
                msg!("Performing addition: {} + {}", input.a, input.b);
                input.a.checked_add(input.b)
                    .ok_or(MathError::Overflow)?
            },
            MathOperation::Subtract => {
                msg!("Performing subtraction: {} - {}", input.a, input.b);
                input.a.checked_sub(input.b)
                    .ok_or(MathError::Underflow)?
            },
            MathOperation::Multiply => {
                msg!("Performing multiplication: {} * {}", input.a, input.b);
                input.a.checked_mul(input.b)
                    .ok_or(MathError::Overflow)?
            },
            MathOperation::Divide => {
                msg!("Performing division: {} / {}", input.a, input.b);
                if input.b == 0 {
                    return Err(MathError::DivisionByZero.into());
                }
                input.a / input.b
            },
        };
        
        msg!("Result: {}", result);
        
        // In a real implementation, you might:
        // - Store the result in an account
        // - Emit an event with the result
        // - Update computation statistics
        
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Process<'info> {
    /// The shard that called this function
    /// CHECK: Validated by the shard program
    pub shard: AccountInfo<'info>,
    
    /// The user who initiated the function call
    pub user: Signer<'info>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug)]
pub struct MathInput {
    pub a: u64,
    pub b: u64,
    pub operation: MathOperation,
}

#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, Copy)]
pub enum MathOperation {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[error_code]
pub enum MathError {
    #[msg("Math overflow occurred")]
    Overflow,
    #[msg("Math underflow occurred")]
    Underflow,
    #[msg("Division by zero")]
    DivisionByZero,
}