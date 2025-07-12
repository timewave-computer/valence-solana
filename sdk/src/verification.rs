//! Verifier interaction helpers

use anchor_lang::prelude::*;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

/// Register a verifier
pub fn build_register_verifier_instruction(
    verifier_program: Pubkey,
    authority: Pubkey,
    label: String,
    program: Pubkey,
) -> Result<Instruction> {
    // Build instruction data
    let mut data = vec![0]; // Register verifier discriminator
    data.extend_from_slice(&(label.len() as u32).to_le_bytes());
    data.extend_from_slice(label.as_bytes());
    data.extend_from_slice(&program.to_bytes());
    
    // Calculate verifier entry PDA
    let (verifier_entry, _) = Pubkey::find_program_address(
        &[b"verifier", label.as_bytes()],
        &verifier_program,
    );
    
    Ok(Instruction {
        program_id: verifier_program,
        accounts: vec![
            AccountMeta::new(authority, true),
            AccountMeta::new(verifier_entry, false),
            AccountMeta::new_readonly(solana_sdk::system_program::id(), false),
        ],
        data,
    })
}

/// Build verification request
pub fn build_verify_predicate_instruction(
    verifier_program: Pubkey,
    caller: Pubkey,
    label: String,
    predicate_data: Vec<u8>,
    context: Vec<u8>,
) -> Result<Instruction> {
    // Build instruction data
    let mut data = vec![2]; // Verify predicate discriminator
    data.extend_from_slice(&(label.len() as u32).to_le_bytes());
    data.extend_from_slice(label.as_bytes());
    data.extend_from_slice(&(predicate_data.len() as u32).to_le_bytes());
    data.extend_from_slice(&predicate_data);
    data.extend_from_slice(&(context.len() as u32).to_le_bytes());
    data.extend_from_slice(&context);
    
    // Calculate verifier entry PDA
    let (verifier_entry, _) = Pubkey::find_program_address(
        &[b"verifier", label.as_bytes()],
        &verifier_program,
    );
    
    Ok(Instruction {
        program_id: verifier_program,
        accounts: vec![
            AccountMeta::new_readonly(caller, true),
            AccountMeta::new_readonly(verifier_entry, false),
            AccountMeta::new_readonly(verifier_program, false),
        ],
        data,
    })
}