//! Registry interaction helpers

use anchor_lang::prelude::*;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    hash::hash,
};

/// Register a function in the global registry
pub fn build_register_instruction(
    registry_program: Pubkey,
    authority: Pubkey,
    hash: [u8; 32],
    program: Pubkey,
) -> Result<Instruction> {
    // Build instruction data
    let mut data = vec![0]; // Register discriminator
    data.extend_from_slice(&hash);
    data.extend_from_slice(&program.to_bytes());
    
    // Calculate function entry PDA
    let (function_entry, _) = Pubkey::find_program_address(
        &[b"function", &hash],
        &registry_program,
    );
    
    Ok(Instruction {
        program_id: registry_program,
        accounts: vec![
            AccountMeta::new(authority, true),
            AccountMeta::new(function_entry, false),
            AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
        ],
        data,
    })
}

/// Generate function hash from content
pub fn generate_function_hash(content: &[u8]) -> [u8; 32] {
    hash(content).to_bytes()
}

/// Build unregister instruction
pub fn build_unregister_instruction(
    registry_program: Pubkey,
    authority: Pubkey,
    hash: [u8; 32],
) -> Result<Instruction> {
    // Build instruction data
    let mut data = vec![1]; // Unregister discriminator
    data.extend_from_slice(&hash);
    
    // Calculate function entry PDA
    let (function_entry, _) = Pubkey::find_program_address(
        &[b"function", &hash],
        &registry_program,
    );
    
    Ok(Instruction {
        program_id: registry_program,
        accounts: vec![
            AccountMeta::new(authority, true),
            AccountMeta::new(function_entry, false),
        ],
        data,
    })
}

