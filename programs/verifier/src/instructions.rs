//! Verifier instructions - Permission checks & routing

use anchor_lang::prelude::*;
use crate::{VerifierEntry, VerificationError};

/// Register a new verifier
pub fn register_verifier(
    ctx: Context<RegisterVerifier>,
    label: String,
    program: Pubkey,
) -> Result<()> {
    // Validate inputs
    require!(
        !label.is_empty() && label.len() <= 32,
        VerificationError::InvalidLabel
    );
    require!(
        program != Pubkey::default(),
        VerificationError::InvalidProgram
    );

    // Initialize verifier entry
    let verifier = &mut ctx.accounts.verifier_entry;
    verifier.label = label.clone();
    verifier.program = program;
    verifier.authority = ctx.accounts.authority.key();
    verifier.registered_at = Clock::get()?.unix_timestamp;

    msg!("Registered verifier: {} at {}", label, program);
    Ok(())
}

/// Update verifier program
pub fn update_verifier(
    ctx: Context<UpdateVerifier>,
    label: String,
    new_program: Pubkey,
) -> Result<()> {
    let verifier = &mut ctx.accounts.verifier_entry;
    
    // Verify label matches
    require!(
        verifier.label == label,
        VerificationError::LabelMismatch
    );
    
    // Verify authority
    require!(
        verifier.authority == ctx.accounts.authority.key(),
        VerificationError::Unauthorized
    );
    
    // Validate new program
    require!(
        new_program != Pubkey::default(),
        VerificationError::InvalidProgram
    );

    let old_program = verifier.program;
    verifier.program = new_program;

    msg!("Updated verifier {} from {} to {}", label, old_program, new_program);
    Ok(())
}

/// Route verification request to appropriate verifier
pub fn verify_predicate(
    ctx: Context<VerifyPredicate>,
    label: String,
    predicate_data: Vec<u8>,
    context: Vec<u8>,
) -> Result<()> {
    let verifier = &ctx.accounts.verifier_entry;
    
    // Verify label matches
    require!(
        verifier.label == label,
        VerificationError::LabelMismatch
    );

    msg!("Routing to verifier: {} at {}", label, verifier.program);
    msg!("Predicate data: {} bytes, Context: {} bytes", 
        predicate_data.len(), context.len());

    // Build instruction data for verifier
    let mut ix_data = vec![0]; // Verify instruction discriminator
    ix_data.extend_from_slice(&(predicate_data.len() as u32).to_le_bytes());
    ix_data.extend_from_slice(&predicate_data);
    ix_data.extend_from_slice(&(context.len() as u32).to_le_bytes());
    ix_data.extend_from_slice(&context);
    
    // Prepare accounts for CPI
    let accounts = vec![
        ctx.accounts.caller.to_account_info(),
        ctx.accounts.verifier_program.to_account_info(),
    ];
    
    // Create instruction for CPI to verifier
    let ix = anchor_lang::solana_program::instruction::Instruction {
        program_id: verifier.program,
        accounts: vec![
            anchor_lang::solana_program::instruction::AccountMeta::new_readonly(
                ctx.accounts.caller.key(),
                true,
            ),
        ],
        data: ix_data,
    };
    
    // Invoke CPI to verifier program
    anchor_lang::solana_program::program::invoke(&ix, &accounts)?;
    
    msg!("Verification completed successfully");
    Ok(())
}

// Account contexts

#[derive(Accounts)]
#[instruction(label: String)]
pub struct RegisterVerifier<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        init,
        payer = authority,
        space = 8 + 32 + 32 + 32 + 8, // discriminator + label + program + authority + timestamp
        seeds = [b"verifier", label.as_bytes()],
        bump,
    )]
    pub verifier_entry: Account<'info, VerifierEntry>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(label: String)]
pub struct UpdateVerifier<'info> {
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"verifier", label.as_bytes()],
        bump,
    )]
    pub verifier_entry: Account<'info, VerifierEntry>,
}

#[derive(Accounts)]
#[instruction(label: String)]
pub struct VerifyPredicate<'info> {
    pub caller: Signer<'info>,
    
    #[account(
        seeds = [b"verifier", label.as_bytes()],
        bump,
    )]
    pub verifier_entry: Account<'info, VerifierEntry>,
    
    /// CHECK: Verifier program will be validated during CPI
    pub verifier_program: UncheckedAccount<'info>,
}