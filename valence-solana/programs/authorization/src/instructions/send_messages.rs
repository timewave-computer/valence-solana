use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::solana_program::program_option::COption;
use anchor_lang::solana_program::system_instruction;
use crate::state::*;
use crate::error::AuthorizationError;
use crate::cache::{AuthorizationCache, helpers};

// Thread-local storage for the authorization cache
thread_local! {
    static AUTHORIZATION_CACHE: RefCell<AuthorizationCache> = RefCell::new(AuthorizationCache::new());
}

pub fn handler(
    ctx: Context<SendMessages>,
    authorization_label: String,
    messages: Vec<ProcessorMessage>,
) -> Result<()> {
    // Check message count
    if messages.is_empty() {
        return Err(error!(AuthorizationError::InvalidMessageFormat));
    }
    if messages.len() > 20 {
        return Err(error!(AuthorizationError::TooManyMessages));
    }
    
    // Update cache with the current authorization for future lookups
    AUTHORIZATION_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        cache.add_authorization(&ctx.accounts.authorization);
    });
    
    // Increment execution counter and update authorization state
    let state = &mut ctx.accounts.authorization_state;
    let execution_id = state.execution_counter;
    state.execution_counter = state.execution_counter.saturating_add(1);
    
    let authorization = &mut ctx.accounts.authorization;
    authorization.current_executions = authorization.current_executions.saturating_add(1);
    
    // Initialize execution tracking
    let execution = &mut ctx.accounts.execution;
    execution.id = execution_id;
    execution.authorization_label = authorization_label.clone();
    execution.sender = ctx.accounts.sender.key();
    execution.start_time = Clock::get()?.unix_timestamp;
    execution.bump = *ctx.bumps.get("execution").unwrap();
    
    // Prepare accounts for the processor
    let mut processor_accounts = vec![
        AccountMeta {
            pubkey: ctx.accounts.processor_program.key(),
            is_signer: false,
            is_writable: false,
        },
        AccountMeta {
            pubkey: ctx.accounts.execution.key(),
            is_signer: false,
            is_writable: true,
        },
        AccountMeta {
            pubkey: ctx.accounts.authorization.key(),
            is_signer: false,
            is_writable: false,
        },
        AccountMeta {
            pubkey: ctx.accounts.sender.key(),
            is_signer: true,
            is_writable: true,
        },
        AccountMeta {
            pubkey: ctx.program_id.key(),
            is_signer: false,
            is_writable: false,
        },
    ];
    
    // Build processor instruction data
    let mut instruction_data = execution_id.to_le_bytes().to_vec();
    
    // Add priority and execution type
    instruction_data.push(priority_to_u8(&authorization.priority));
    instruction_data.push(subroutine_type_to_u8(&authorization.subroutine_type));
    
    // Serialize messages
    let messages_data = messages
        .try_to_vec()
        .map_err(|_| error!(AuthorizationError::InvalidMessageFormat))?;
    instruction_data.extend_from_slice(&messages_data);
    
    // Send the instruction to the processor
    let processor_ix = Instruction {
        program_id: ctx.accounts.processor_program.key(),
        accounts: processor_accounts,
        data: instruction_data,
    };
    
    invoke(
        &processor_ix,
        &[
            ctx.accounts.processor_program.to_account_info(),
            ctx.accounts.execution.to_account_info(),
            ctx.accounts.authorization.to_account_info(),
            ctx.accounts.sender.to_account_info(),
            ctx.accounts.authorization_state.to_account_info(),
        ],
    )?;
    
    msg!("Messages sent through authorization {}: {}", authorization_label, execution_id);
    
    Ok(())
}

fn priority_to_u8(priority: &Priority) -> u8 {
    match priority {
        Priority::Low => 0,
        Priority::Medium => 1,
        Priority::High => 2,
    }
}

fn subroutine_type_to_u8(subroutine_type: &SubroutineType) -> u8 {
    match subroutine_type {
        SubroutineType::Atomic => 0,
        SubroutineType::NonAtomic => 1,
    }
} 