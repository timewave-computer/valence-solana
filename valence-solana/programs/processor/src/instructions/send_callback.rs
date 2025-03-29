use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke;
use anchor_lang::solana_program::instruction::Instruction;
use crate::state::*;
use crate::error::ProcessorError;

pub fn handler(
    ctx: Context<SendCallback>,
    execution_id: u64,
    result: ExecutionResult,
    executed_count: u32,
    error_data: Option<Vec<u8>>,
) -> Result<()> {
    // Get pending callback
    let pending_callback = &ctx.accounts.pending_callback;
    
    // Verify execution ID matches
    if pending_callback.execution_id != execution_id {
        return Err(error!(ProcessorError::NoPendingCallback));
    }
    
    // Prepare accounts for the authorization program
    let mut accounts = vec![
        AccountMeta {
            pubkey: ctx.accounts.authorization_program.key(),
            is_signer: false,
            is_writable: false,
        },
        AccountMeta {
            pubkey: ctx.accounts.callback_recipient.key(),
            is_signer: false,
            is_writable: true,
        },
        AccountMeta {
            pubkey: pending_callback.callback_address,
            is_signer: false,
            is_writable: true,
        },
        AccountMeta {
            pubkey: ctx.accounts.fee_payer.key(),
            is_signer: true,
            is_writable: true,
        },
    ];
    
    // Build instruction data
    let mut instruction_data = execution_id.to_le_bytes().to_vec();
    
    // Add result code
    let result_code = match result {
        ExecutionResult::Success => 0u8,
        ExecutionResult::Failure => 1u8,
    };
    instruction_data.push(result_code);
    
    // Add executed count
    instruction_data.extend_from_slice(&executed_count.to_le_bytes());
    
    // Add error data if present
    if let Some(error_data) = &error_data {
        instruction_data.extend_from_slice(error_data);
    }
    
    // Create the instruction
    let receive_callback_ix = Instruction {
        program_id: ctx.accounts.authorization_program.key(),
        accounts,
        data: instruction_data,
    };
    
    // Invoke the instruction
    // This is a stub - in a real implementation, we would invoke the authorization program
    /*
    invoke(
        &receive_callback_ix,
        &[
            ctx.accounts.authorization_program.to_account_info(),
            ctx.accounts.callback_recipient.to_account_info(),
            ctx.accounts.pending_callback.to_account_info(),
            ctx.accounts.fee_payer.to_account_info(),
        ],
    )?;
    */
    
    // Log the callback
    msg!(
        "Sending callback for execution_id: {}, result: {:?}, executed_count: {}",
        execution_id,
        result,
        executed_count
    );
    
    // Note: The pending callback account is closed in the account validation,
    // and the rent is returned to the fee payer
    
    Ok(())
} 