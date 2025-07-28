//! ZK Transfer Limit Example
//! 
//! This example demonstrates how to use the verification gateway guard
//! to enforce transfer limits on borrowed accounts using zero-knowledge proofs.
//!
//! The ZK circuit proves "this transaction transfers at most 10 SOL"
//! without revealing the actual transfer amount on-chain.

use anchor_lang::prelude::*;
use anchor_lang::system_program;
use valence_kernel::{Guard, guards::SerializedGuard, operations::{SessionOperation, OperationBatch}};
use valence_functions::guards::zk_verification_gateway::{
    ZkGuardData, ProofSystem, GuardEvaluationContext
};
use valence_sdk::{compile_guard, SessionBuilder};
use borsh::{BorshSerialize, BorshDeserialize};

pub const TRANSFER_LIMIT_REGISTRY_ID: u64 = 100;
pub const MAX_TRANSFER_LAMPORTS: u64 = 10 * 1_000_000_000; // 10 SOL

/// Public values for the transfer limit circuit
#[derive(BorshSerialize, BorshDeserialize, Debug)]
pub struct TransferLimitPublicValues {
    /// Hash of the transaction being authorized
    pub transaction_hash: [u8; 32],
    /// Maximum allowed transfer (10 SOL)
    pub max_transfer: u64,
    /// The source account that will be debited
    pub source_account: Pubkey,
    /// The destination account
    pub destination_account: Pubkey,
}

/// Setup the ZK transfer limit verification
pub async fn setup_transfer_limit_guard(
    client: &anchor_client::Client,
    admin: &Keypair,
    authorized_users: Vec<Pubkey>,
) -> Result<()> {
    // 1. Generate verification key for the transfer limit circuit
    // In practice, this would come from your ZK circuit compilation
    let verification_key = generate_transfer_limit_vk();
    
    // 2. Register the verification key
    register_transfer_limit_vk(
        client,
        admin,
        TRANSFER_LIMIT_REGISTRY_ID,
        verification_key,
        authorized_users, // Only these users can submit proofs
    ).await?;
    
    println!("✓ Registered transfer limit verification key");
    
    Ok(())
}

/// Create a session that can transfer up to 10 SOL from a borrowed account
pub async fn create_transfer_limit_session(
    client: &anchor_client::Client,
    owner: &Keypair,
    source_account: Pubkey,
    transfer_proof: Vec<u8>,
    public_values: TransferLimitPublicValues,
) -> Result<Pubkey> {
    // 1. Create guard data for ZK verification
    let guard_data = ZkGuardData {
        vk_id: TRANSFER_LIMIT_REGISTRY_ID,
        proof_system: ProofSystem::SP1,
        proof: transfer_proof,
        public_values: public_values.try_to_vec()?,
        require_whitelisted_submitter: true, // Only authorized users
    };
    
    // 2. Create the external guard
    let guard = Guard::External {
        program: valence_functions::guards::zk_verification_gateway::ID,
        data: guard_data.try_to_vec()?,
        required_accounts: vec![
            // VK PDA
            AccountMeta::new_readonly(
                derive_vk_pda(TRANSFER_LIMIT_REGISTRY_ID, owner.pubkey()),
                false
            ),
            // Verifier programs
            AccountMeta::new_readonly(sp1_verifier_id(), false),
            AccountMeta::new_readonly(groth16_verifier_id(), false),
            AccountMeta::new_readonly(plonk_verifier_id(), false),
            AccountMeta::new_readonly(halo2_verifier_id(), false),
            // Payer for whitelist check
            AccountMeta::new_readonly(owner.pubkey(), false),
        ],
    };
    
    // 3. Compile and create session
    let serialized_guard = compile_guard(&guard)?;
    
    let session = SessionBuilder::new()
        .with_owner(owner.pubkey())
        .with_guard(serialized_guard)
        .with_borrowed_account(source_account) // Borrow the source account
        .build(client)
        .await?;
    
    println!("✓ Created session with transfer limit guard");
    println!("  Session: {}", session);
    println!("  Borrowed account: {}", source_account);
    
    Ok(session)
}

/// Execute a transfer with ZK proof of amount limit
pub async fn execute_limited_transfer(
    client: &anchor_client::Client,
    session: Pubkey,
    source: Pubkey,
    destination: Pubkey,
    amount_lamports: u64,
) -> Result<()> {
    // Create the transfer instruction
    let transfer_ix = system_program::transfer(
        &source,
        &destination,
        amount_lamports,
    );
    
    // Wrap in session operation
    let operation = SessionOperation::InvokeProgram {
        program_id: system_program::ID,
        accounts: vec![
            AccountMeta::new(source, false), // Will be borrowed from session
            AccountMeta::new(destination, false),
        ],
        data: transfer_ix.data,
    };
    
    let batch = OperationBatch {
        operations: vec![operation],
    };
    
    // Execute - the guard will verify the ZK proof
    valence_sdk::execute_session_operations(
        client,
        session,
        batch,
    ).await?;
    
    println!("✓ Transfer executed with ZK verification");
    println!("  Amount: {} lamports ({} SOL)", amount_lamports, amount_lamports as f64 / 1e9);
    
    Ok(())
}

/// Generate a ZK proof that a transfer is within limits
pub fn generate_transfer_proof(
    source: Pubkey,
    destination: Pubkey,
    amount_lamports: u64,
) -> Result<(Vec<u8>, TransferLimitPublicValues)> {
    // In a real implementation, this would:
    // 1. Take the transfer details as private input
    // 2. Prove that amount_lamports <= MAX_TRANSFER_LAMPORTS
    // 3. Commit to the transaction hash
    
    require!(
        amount_lamports <= MAX_TRANSFER_LAMPORTS,
        "Transfer amount exceeds limit"
    );
    
    // Public values visible to the verifier
    let public_values = TransferLimitPublicValues {
        transaction_hash: hash_transfer(source, destination, amount_lamports),
        max_transfer: MAX_TRANSFER_LAMPORTS,
        source_account: source,
        destination_account: destination,
    };
    
    // Simulated proof (in practice, generated by ZK prover)
    let proof = vec![42u8; 260]; // SP1 proofs are ~260 bytes
    
    Ok((proof, public_values))
}

// ===== Helper Functions =====

fn derive_vk_pda(registry_id: u64, owner: Pubkey) -> Pubkey {
    let (pda, _) = Pubkey::find_program_address(
        &[
            b"vk",
            registry_id.to_le_bytes().as_ref(),
            owner.as_ref(),
        ],
        &valence_functions::guards::zk_verification_gateway::ID,
    );
    pda
}

async fn register_transfer_limit_vk(
    client: &anchor_client::Client,
    admin: &Keypair,
    registry_id: u64,
    verification_key: Vec<u8>,
    whitelisted_submitters: Vec<Pubkey>,
) -> Result<()> {
    let program = client.program(valence_functions::guards::zk_verification_gateway::ID);
    
    program
        .request()
        .accounts(valence_functions::guards::zk_verification_gateway::accounts::RegisterVK {
            verification_key: derive_vk_pda(registry_id, admin.pubkey()),
            owner: admin.pubkey(),
            admin: admin.pubkey(),
            payer: admin.pubkey(),
            system_program: system_program::ID,
        })
        .args(valence_functions::guards::zk_verification_gateway::instruction::RegisterVerificationKey {
            registry_id,
            proof_system: ProofSystem::SP1,
            verification_key,
            whitelisted_submitters,
        })
        .signer(admin)
        .send()
        .await?;
    
    Ok(())
}

fn generate_transfer_limit_vk() -> Vec<u8> {
    // In practice, this would come from compiling your ZK circuit
    // that proves: amount <= MAX_TRANSFER_LAMPORTS
    vec![0u8; 32]
}

fn hash_transfer(source: Pubkey, destination: Pubkey, amount: u64) -> [u8; 32] {
    use anchor_lang::solana_program::keccak;
    let data = [
        source.as_ref(),
        destination.as_ref(),
        &amount.to_le_bytes(),
    ].concat();
    keccak::hash(&data).to_bytes()
}

// Placeholder verifier program IDs
fn sp1_verifier_id() -> Pubkey {
    // In practice, this would be the deployed SP1 verifier
    Pubkey::new_from_array([1u8; 32])
}

fn groth16_verifier_id() -> Pubkey {
    Pubkey::new_from_array([2u8; 32])
}

fn plonk_verifier_id() -> Pubkey {
    Pubkey::new_from_array([3u8; 32])
}

fn halo2_verifier_id() -> Pubkey {
    Pubkey::new_from_array([4u8; 32])
}

// ===== Tests =====

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_vk_pda_derivation() {
        let owner = Pubkey::new_unique();
        let registry_id = 42u64;
        
        let pda1 = derive_vk_pda(registry_id, owner);
        let pda2 = derive_vk_pda(registry_id, owner);
        
        assert_eq!(pda1, pda2);
    }
    
    #[test]
    fn test_transfer_limit_validation() {
        let source = Pubkey::new_unique();
        let dest = Pubkey::new_unique();
        
        // Should succeed - under limit
        let result = generate_transfer_proof(source, dest, 5 * 1_000_000_000);
        assert!(result.is_ok());
        
        // Should fail - over limit
        let result = generate_transfer_proof(source, dest, 15 * 1_000_000_000);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_public_values_serialization() {
        let public_values = TransferLimitPublicValues {
            transaction_hash: [1u8; 32],
            max_transfer: MAX_TRANSFER_LAMPORTS,
            source_account: Pubkey::new_unique(),
            destination_account: Pubkey::new_unique(),
        };
        
        let serialized = public_values.try_to_vec().unwrap();
        let deserialized: TransferLimitPublicValues = 
            BorshDeserialize::try_from_slice(&serialized).unwrap();
        
        assert_eq!(deserialized.max_transfer, MAX_TRANSFER_LAMPORTS);
        assert_eq!(deserialized.transaction_hash, [1u8; 32]);
    }
}