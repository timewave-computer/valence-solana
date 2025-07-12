//! Gateway routing helpers

use anchor_lang::prelude::*;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

/// Target for gateway routing
#[derive(Debug, Clone)]
pub enum RouteTarget {
    Registry,
    Verifier,
    Shard(Pubkey),
}

/// Build gateway routing instruction
pub fn build_route_instruction(
    gateway_program: Pubkey,
    signer: Pubkey,
    target: RouteTarget,
    data: Vec<u8>,
) -> Result<Instruction> {
    // Build instruction data based on target
    let ix_data = match target {
        RouteTarget::Registry => {
            let mut data = vec![0]; // Route discriminator
            data.extend_from_slice(&[0]); // Registry target
            data
        }
        RouteTarget::Verifier => {
            let mut data = vec![0]; // Route discriminator
            data.extend_from_slice(&[1]); // Verifier target
            data
        }
        RouteTarget::Shard(shard_id) => {
            let mut data = vec![0]; // Route discriminator
            data.extend_from_slice(&[2]); // Shard target
            data.extend_from_slice(&shard_id.to_bytes());
            data
        }
    };
    
    // Add the actual instruction data
    let mut final_data = ix_data;
    final_data.extend_from_slice(&data);
    
    Ok(Instruction {
        program_id: gateway_program,
        accounts: vec![
            AccountMeta::new(signer, true),
        ],
        data: final_data,
    })
}