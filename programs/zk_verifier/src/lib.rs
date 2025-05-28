// ZK Proof Verifier Program for Valence Protocol
// Simplified implementation matching Solidity VerificationGateway pattern

use anchor_lang::prelude::*;

declare_id!("5xk7TofwN46GUpkRoLAtJVaGkfHGYY7wm3aGWAzBAmq7");

pub mod error;
pub mod state;
pub mod instructions;

use state::*;


#[program]
pub mod zk_verifier {
    use super::*;
    
    /// Initialize the ZK verifier program
    pub fn initialize(
        ctx: Context<Initialize>,
        coprocessor_root: [u8; 32],
        verifier: Pubkey,
    ) -> Result<()> {
        instructions::initialize::handler(ctx, coprocessor_root, verifier)
    }
    
    /// Add a verification key for a registry ID (like Solidity addRegistry)
    pub fn add_registry(
        ctx: Context<AddRegistry>,
        registry_id: u64,
        vk_hash: [u8; 32],
        key_type: VerificationKeyType,
    ) -> Result<()> {
        instructions::add_registry::handler(ctx, registry_id, vk_hash, key_type)
    }
    
    /// Remove a verification key for a registry ID (like Solidity removeRegistry)
    pub fn remove_registry(
        ctx: Context<RemoveRegistry>,
        registry_id: u64,
    ) -> Result<()> {
        instructions::remove_registry::handler(ctx, registry_id)
    }
    
    /// Verify a ZK proof (like Solidity verify function)
    pub fn verify(
        ctx: Context<Verify>,
        registry_id: u64,
        proof: Vec<u8>,
        message: Vec<u8>,
    ) -> Result<bool> {
        instructions::verify::handler(ctx, registry_id, proof, message)
    }
}

// Account structs for the program instructions
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = owner,
        space = VerifierState::SPACE,
        seeds = [b"verifier_state"],
        bump
    )]
    pub verifier_state: Account<'info, VerifierState>,
    
    #[account(mut)]
    pub owner: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(registry_id: u64)]
pub struct AddRegistry<'info> {
    #[account(
        mut,
        seeds = [b"verifier_state"],
        bump = verifier_state.bump,
        constraint = verifier_state.owner == owner.key(),
    )]
    pub verifier_state: Account<'info, VerifierState>,
    
    #[account(
        init,
        payer = owner,
        space = VerificationKey::SPACE,
        seeds = [b"verification_key", owner.key().as_ref(), &registry_id.to_le_bytes()],
        bump
    )]
    pub verification_key: Account<'info, VerificationKey>,
    
    #[account(mut)]
    pub owner: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(registry_id: u64)]
pub struct RemoveRegistry<'info> {
    #[account(
        mut,
        seeds = [b"verifier_state"],
        bump = verifier_state.bump,
        constraint = verifier_state.owner == owner.key(),
    )]
    pub verifier_state: Account<'info, VerifierState>,
    
    #[account(
        mut,
        seeds = [b"verification_key", owner.key().as_ref(), &registry_id.to_le_bytes()],
        bump = verification_key.bump,
        close = owner
    )]
    pub verification_key: Account<'info, VerificationKey>,
    
    #[account(mut)]
    pub owner: Signer<'info>,
    
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(registry_id: u64)]
pub struct Verify<'info> {
    #[account(
        seeds = [b"verification_key", program_owner.key().as_ref(), &registry_id.to_le_bytes()],
        bump = verification_key.bump,
        constraint = verification_key.is_active @ error::VerifierError::VerificationKeyInactive,
    )]
    pub verification_key: Account<'info, VerificationKey>,
    
    /// CHECK: This is the program owner that registered the verification key
    pub program_owner: AccountInfo<'info>,
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_verification_key_type_serialization() {
        let key_types = vec![
            VerificationKeyType::SP1,
            VerificationKeyType::Groth16,
            VerificationKeyType::PLONK,
        ];

        for key_type in key_types {
            let serialized = key_type.try_to_vec().unwrap();
            let deserialized: VerificationKeyType = VerificationKeyType::try_from_slice(&serialized).unwrap();
            assert_eq!(deserialized, key_type);
        }
    }

    #[test]
    fn test_verifier_state_space() {
        let space = VerifierState::SPACE;
        assert!(space > 50 && space < 200);
    }

    #[test]
    fn test_verification_key_space() {
        let space = VerificationKey::SPACE;
        assert!(space > 50 && space < 200);
    }
} 