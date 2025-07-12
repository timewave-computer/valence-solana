//! Gateway instructions - Route to registry/verification/shards

use anchor_lang::prelude::*;
use crate::{RegistryInstruction, VerificationInstruction};

/// Route to registry program
pub fn route_to_registry(
    ctx: Context<RouteToRegistry>,
    instruction: RegistryInstruction,
) -> Result<()> {
    msg!("Gateway: Routing to registry - {:?}", instruction);
    
    // Build instruction data based on registry instruction type
    let ix_data = match instruction {
        RegistryInstruction::Register { hash, program } => {
            // Serialize register instruction
            let mut data = vec![0]; // Instruction discriminator for register
            data.extend_from_slice(&hash);
            data.extend_from_slice(&program.to_bytes());
            data
        }
        RegistryInstruction::Unregister { hash } => {
            // Serialize unregister instruction
            let mut data = vec![1]; // Instruction discriminator for unregister
            data.extend_from_slice(&hash);
            data
        }
    };
    
    // Prepare accounts for CPI
    let accounts = vec![
        ctx.accounts.signer.to_account_info(),
        ctx.accounts.registry_program.to_account_info(),
    ];
    
    // Create instruction for CPI
    let ix = anchor_lang::solana_program::instruction::Instruction {
        program_id: ctx.accounts.registry_program.key(),
        accounts: vec![
            anchor_lang::solana_program::instruction::AccountMeta::new(
                ctx.accounts.signer.key(),
                true,
            ),
        ],
        data: ix_data,
    };
    
    // Invoke CPI
    anchor_lang::solana_program::program::invoke(&ix, &accounts)?;
    
    Ok(())
}

/// Route to verifier
pub fn route_to_verification(
    ctx: Context<RouteToVerification>,
    instruction: VerificationInstruction,
) -> Result<()> {
    msg!("Gateway: Routing to verifier - {:?}", instruction);
    
    // Build instruction data based on verification instruction type
    let ix_data = match instruction {
        VerificationInstruction::RegisterVerifier { label, program } => {
            // Serialize register verifier instruction
            let mut data = vec![0]; // Instruction discriminator
            data.extend_from_slice(&(label.len() as u32).to_le_bytes());
            data.extend_from_slice(label.as_bytes());
            data.extend_from_slice(&program.to_bytes());
            data
        }
        VerificationInstruction::UpdateVerifier { label, new_program } => {
            // Serialize update verifier instruction
            let mut data = vec![1]; // Instruction discriminator
            data.extend_from_slice(&(label.len() as u32).to_le_bytes());
            data.extend_from_slice(label.as_bytes());
            data.extend_from_slice(&new_program.to_bytes());
            data
        }
        VerificationInstruction::VerifyPredicate { label, predicate_data, context } => {
            // Serialize verify predicate instruction
            let mut data = vec![2]; // Instruction discriminator
            data.extend_from_slice(&(label.len() as u32).to_le_bytes());
            data.extend_from_slice(label.as_bytes());
            data.extend_from_slice(&(predicate_data.len() as u32).to_le_bytes());
            data.extend_from_slice(&predicate_data);
            data.extend_from_slice(&(context.len() as u32).to_le_bytes());
            data.extend_from_slice(&context);
            data
        }
    };
    
    // Prepare accounts for CPI
    let accounts = vec![
        ctx.accounts.signer.to_account_info(),
        ctx.accounts.verification_program.to_account_info(),
    ];
    
    // Create instruction for CPI
    let ix = anchor_lang::solana_program::instruction::Instruction {
        program_id: ctx.accounts.verification_program.key(),
        accounts: vec![
            anchor_lang::solana_program::instruction::AccountMeta::new(
                ctx.accounts.signer.key(),
                true,
            ),
        ],
        data: ix_data,
    };
    
    // Invoke CPI
    anchor_lang::solana_program::program::invoke(&ix, &accounts)?;
    
    Ok(())
}

/// Route to a specific shard
pub fn route_to_shard(
    ctx: Context<RouteToShard>,
    shard_id: Pubkey,
    instruction_data: Vec<u8>,
) -> Result<()> {
    msg!("Gateway: Routing to shard {} - {} bytes", shard_id, instruction_data.len());
    
    // Verify shard_id matches the provided program
    require!(
        shard_id == ctx.accounts.shard_program.key(),
        crate::GatewayError::InvalidTarget
    );
    
    // Prepare accounts for CPI to shard
    let accounts = vec![
        ctx.accounts.signer.to_account_info(),
        ctx.accounts.shard_program.to_account_info(),
    ];
    
    // Create instruction for CPI
    let ix = anchor_lang::solana_program::instruction::Instruction {
        program_id: shard_id,
        accounts: vec![
            anchor_lang::solana_program::instruction::AccountMeta::new(
                ctx.accounts.signer.key(),
                true,
            ),
        ],
        data: instruction_data,
    };
    
    // Invoke CPI to shard
    anchor_lang::solana_program::program::invoke(&ix, &accounts)?;
    
    Ok(())
}

#[derive(Accounts)]
pub struct RouteToRegistry<'info> {
    pub signer: Signer<'info>,
    /// CHECK: Registry program will be validated during CPI
    pub registry_program: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct RouteToVerification<'info> {
    pub signer: Signer<'info>,
    /// CHECK: Verifier program will be validated during CPI
    pub verification_program: UncheckedAccount<'info>,
}

#[derive(Accounts)]
pub struct RouteToShard<'info> {
    pub signer: Signer<'info>,
    /// CHECK: Shard program will be validated during CPI
    pub shard_program: UncheckedAccount<'info>,
}