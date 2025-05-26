// Verify SMT membership instruction for ZK Proof Verifier Program

use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::VerifierError;

pub fn handler(
    ctx: Context<VerifySMTMembership>,
    smt_proof: SMTProof,
) -> Result<bool> {
    let smt_state = &ctx.accounts.smt_state;
    
    // Verify the root matches the SMT state
    if smt_proof.root != smt_state.root {
        msg!("SMT root mismatch: expected {:?}, got {:?}", smt_state.root, smt_proof.root);
        return Ok(false);
    }
    
    // Verify the merkle path
    let is_valid = verify_merkle_path(&smt_proof)?;
    
    if is_valid {
        msg!("SMT membership proof verified for key: {:?}", smt_proof.key);
    } else {
        msg!("SMT membership proof failed for key: {:?}", smt_proof.key);
    }
    
    Ok(is_valid)
}

/// Verify the merkle path in an SMT proof
fn verify_merkle_path(proof: &SMTProof) -> Result<bool> {
    use anchor_lang::solana_program::hash::Hasher;
    
    // Start with the leaf hash
    let mut current_hash = if let Some(value) = proof.value {
        // Membership proof: hash(key, value)
        let mut hasher = Hasher::default();
        hasher.hash(&proof.key);
        hasher.hash(&value);
        hasher.result().to_bytes()
    } else {
        // Non-membership proof: use empty hash for non-existent leaf
        [0u8; 32]
    };
    
    // Verify path length matches directions length
    if proof.path.len() != proof.directions.len() {
        return Err(VerifierError::InvalidProof.into());
    }
    
    // Traverse the path from leaf to root
    for (i, &sibling_hash) in proof.path.iter().enumerate() {
        let direction = proof.directions[i];
        
        // Compute parent hash based on direction
        let mut hasher = Hasher::default();
        if direction {
            // Current node is right child
            hasher.hash(&sibling_hash);
            hasher.hash(&current_hash);
        } else {
            // Current node is left child
            hasher.hash(&current_hash);
            hasher.hash(&sibling_hash);
        }
        current_hash = hasher.result().to_bytes();
    }
    
    // Final hash should match the root
    Ok(current_hash == proof.root)
}

#[derive(Accounts)]
#[instruction(smt_proof: SMTProof)]
pub struct VerifySMTMembership<'info> {
    /// The SMT state account
    #[account(
        seeds = [b"smt_state".as_ref(), smt_state.owner.as_ref()],
        bump = smt_state.bump,
    )]
    pub smt_state: Account<'info, SMTState>,
    
    /// The account requesting verification
    pub verifier: Signer<'info>,
} 