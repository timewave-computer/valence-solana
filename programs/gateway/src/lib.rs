//! Gateway - Entry point router for all Valence operations
//! 
//! This is a thin routing layer that directs operations to:
//! - Registry: Function registration and lookup
//! - Verifier: Permission checks and verification routing
//! - Shards: User-deployed contract instances

use anchor_lang::prelude::*;

declare_id!("11111111111111111111111111111111");

pub mod state;
pub mod instructions;
pub mod error;

pub use state::*;
pub use instructions::*;
pub use error::*;

#[program]
pub mod valence_gateway {
    use super::*;

    /// Route an operation to the appropriate target
    pub fn route(_ctx: Context<Route>, target: RouteTarget, _data: Vec<u8>) -> Result<()> {
        match target {
            RouteTarget::Registry { instruction } => {
                // Route to registry program
                msg!("Routing to registry: {:?}", instruction);
                // CPI to registry program
                Ok(())
            }
            RouteTarget::Verifier { instruction } => {
                // Route to verifier
                msg!("Routing to verifier: {:?}", instruction);
                // CPI to verifier program
                Ok(())
            }
            RouteTarget::Shard { id, instruction_data } => {
                // Route to specific shard
                msg!("Routing to shard {}: {} bytes", id, instruction_data.len());
                // CPI to shard program
                Ok(())
            }
        }
    }
}

#[derive(Accounts)]
pub struct Route<'info> {
    /// Any signer can initiate routing
    pub signer: Signer<'info>,
}

/// Routing targets
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum RouteTarget {
    /// Route to the registry singleton
    Registry { 
        instruction: RegistryInstruction 
    },
    /// Route to the verifier singleton
    Verifier { 
        instruction: VerificationInstruction 
    },
    /// Route to a specific shard
    Shard { 
        id: Pubkey, 
        instruction_data: Vec<u8> 
    },
}

/// Registry instructions that can be routed
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum RegistryInstruction {
    Register { hash: [u8; 32], program: Pubkey },
    Unregister { hash: [u8; 32] },
}

/// Verifier instructions that can be routed
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum VerificationInstruction {
    RegisterVerifier { label: String, program: Pubkey },
    UpdateVerifier { label: String, new_program: Pubkey },
    VerifyPredicate { label: String, predicate_data: Vec<u8>, context: Vec<u8> },
}