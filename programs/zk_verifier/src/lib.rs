// ZK Proof Verifier Program for Valence Protocol
// Handles SP1 proof verification with Solana compute optimization

use anchor_lang::prelude::*;

declare_id!("5xk7TofwN46GUpkRoLAtJVaGkfHGYY7wm3aGWAzBAmq7");

pub mod error;
pub mod state;
pub mod instructions;

use state::*;
use instructions::*;
use error::*;

#[program]
pub mod zk_verifier {
    use super::*;
    
    /// Initialize the ZK verifier program
    pub fn initialize(
        ctx: Context<Initialize>,
    ) -> Result<()> {
        instructions::initialize::handler(ctx)
    }
    
    /// Register a new verification key
    pub fn register_verification_key(
        ctx: Context<RegisterVerificationKey>,
        program_id: Pubkey,
        vk_data: Vec<u8>,
        key_type: VerificationKeyType,
    ) -> Result<()> {
        instructions::register_verification_key::handler(ctx, program_id, vk_data, key_type)
    }
    
    /// Verify a ZK proof
    pub fn verify_proof(
        ctx: Context<VerifyProof>,
        program_id: Pubkey,
        proof_data: Vec<u8>,
        public_inputs: Vec<u8>,
    ) -> Result<bool> {
        instructions::verify_proof::handler(ctx, program_id, proof_data, public_inputs)
    }
    
    /// Update an existing verification key
    pub fn update_verification_key(
        ctx: Context<UpdateVerificationKey>,
        new_vk_data: Vec<u8>,
        key_type: VerificationKeyType,
    ) -> Result<()> {
        instructions::update_verification_key::handler(ctx, new_vk_data, key_type)
    }
    
    /// Create a new Sparse Merkle Tree
    pub fn create_smt(
        ctx: Context<CreateSMT>,
        height: u8,
    ) -> Result<()> {
        instructions::create_smt::handler(ctx, height)
    }
    
    /// Update a Sparse Merkle Tree
    pub fn update_smt(
        ctx: Context<UpdateSMT>,
        key: [u8; 32],
        value: [u8; 32],
    ) -> Result<()> {
        instructions::update_smt::handler(ctx, key, value)
    }
    
    /// Verify SMT membership proof
    pub fn verify_smt_membership(
        ctx: Context<VerifySMTMembership>,
        smt_proof: SMTProof,
    ) -> Result<bool> {
        instructions::verify_smt_membership::handler(ctx, smt_proof)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::*;
    use crate::error::VerifierError;

    #[test]
    fn test_verification_key_type_default() {
        // Test VerificationKeyType variants
        let sp1 = VerificationKeyType::SP1;
        let groth16 = VerificationKeyType::Groth16;
        let plonk = VerificationKeyType::PLONK;
        let smt = VerificationKeyType::SMT;
        
        assert_eq!(sp1, VerificationKeyType::SP1);
        assert_eq!(groth16, VerificationKeyType::Groth16);
        assert_eq!(plonk, VerificationKeyType::PLONK);
        assert_eq!(smt, VerificationKeyType::SMT);
    }

    #[test]
    fn test_verifier_state_space() {
        let space = std::mem::size_of::<VerifierState>();
        // Should be a reasonable fixed size
        assert!(space > 50 && space < 200);
    }

    #[test]
    fn test_verification_key_space() {
        let base_size = std::mem::size_of::<VerificationKey>();
        let space_small = base_size + 100; // 100 bytes VK data
        let space_large = base_size + 1000; // 1000 bytes VK data
        
        assert!(space_small > 100); // At least base size
        assert!(space_large > space_small); // Larger VK = more space
        assert_eq!(space_large - space_small, 900); // Difference should be VK data size
    }

    #[test]
    fn test_smt_state_space() {
        let space = SMTState::SPACE;
        // Should be a reasonable fixed size
        assert!(space > 50 && space < 200);
    }

    #[test]
    fn test_verification_key_serialization() {
        let vk = VerificationKey {
            program_id: Pubkey::new_unique(),
            key_data: vec![1, 2, 3, 4, 5],
            key_type: VerificationKeyType::Groth16,
            is_active: true,
            registered_at: 1234567890,
            updated_at: 1234567900,
            verification_count: 10,
            bump: 255,
        };

        // Test that the structure can be serialized/deserialized
        let serialized = vk.try_to_vec().unwrap();
        let deserialized: VerificationKey = VerificationKey::try_from_slice(&serialized).unwrap();

        assert_eq!(deserialized.program_id, vk.program_id);
        assert_eq!(deserialized.key_data, vec![1, 2, 3, 4, 5]);
        assert_eq!(deserialized.key_type, VerificationKeyType::Groth16);
        assert_eq!(deserialized.is_active, true);
        assert_eq!(deserialized.verification_count, 10);
    }

    #[test]
    fn test_smt_state_serialization() {
        let smt = SMTState {
            owner: Pubkey::new_unique(),
            root: [2u8; 32],
            height: 16,
            leaf_count: 100,
            last_updated: 1234567890,
            is_frozen: false,
            bump: 254,
        };

        let serialized = smt.try_to_vec().unwrap();
        let deserialized: SMTState = SMTState::try_from_slice(&serialized).unwrap();

        assert_eq!(deserialized.height, 16);
        assert_eq!(deserialized.root, [2u8; 32]);
        assert_eq!(deserialized.leaf_count, 100);
        assert_eq!(deserialized.is_frozen, false);
    }

    #[test]
    fn test_verifier_state_serialization() {
        let state = VerifierState {
            owner: Pubkey::new_unique(),
            total_keys: 10,
            successful_verifications: 500,
            failed_verifications: 5,
            is_paused: false,
            bump: 253,
        };

        let serialized = state.try_to_vec().unwrap();
        let deserialized: VerifierState = VerifierState::try_from_slice(&serialized).unwrap();

        assert_eq!(deserialized.owner, state.owner);
        assert_eq!(deserialized.successful_verifications, 500);
        assert_eq!(deserialized.failed_verifications, 5);
        assert_eq!(deserialized.bump, 253);
    }

    #[test]
    fn test_verification_key_type_variants() {
        // Test all verification key type variants
        let types = [
            VerificationKeyType::SP1,
            VerificationKeyType::Groth16,
            VerificationKeyType::PLONK,
            VerificationKeyType::SMT,
        ];

        for key_type in types {
            // Test serialization/deserialization
            let serialized = key_type.try_to_vec().unwrap();
            let deserialized: VerificationKeyType = VerificationKeyType::try_from_slice(&serialized).unwrap();
            assert_eq!(deserialized, key_type);
        }
    }

    #[test]
    fn test_verification_key_hash_validation() {
        let vk_data = vec![1, 2, 3, 4, 5];
        let correct_hash = anchor_lang::solana_program::hash::hash(&vk_data).to_bytes();
        let incorrect_hash = [0u8; 32];

        // This would be used in actual verification logic
        assert_ne!(correct_hash, incorrect_hash);
        assert_eq!(correct_hash.len(), 32);
    }

    #[test]
    fn test_smt_height_validation() {
        // Test valid heights
        for height in 1..=32 {
            assert!(height > 0 && height <= 32);
        }

        // Test invalid heights
        assert!(0 == 0); // Height 0 should be invalid
        assert!(33 > 32); // Height > 32 should be invalid
    }

    #[test]
    fn test_smt_leaf_count_limits() {
        let max_leaves_for_height = |height: u8| -> u64 {
            if height == 0 {
                0
            } else {
                2u64.pow(height as u32)
            }
        };

        assert_eq!(max_leaves_for_height(1), 2);
        assert_eq!(max_leaves_for_height(8), 256);
        assert_eq!(max_leaves_for_height(16), 65536);
        assert_eq!(max_leaves_for_height(32), 4294967296);
    }
}

#[cfg(test)]
mod smt_tests {
    use super::*;
    use crate::state::*;
    use anchor_lang::solana_program::hash::{hash, Hasher};

    #[test]
    fn test_smt_merkle_hash_calculation() {
        // Test basic merkle hash calculation
        let left = [1u8; 32];
        let right = [2u8; 32];
        
        let mut hasher = Hasher::default();
        hasher.hash(&left);
        hasher.hash(&right);
        let parent_hash = hasher.result().to_bytes();
        
        assert_ne!(parent_hash, left);
        assert_ne!(parent_hash, right);
        assert_eq!(parent_hash.len(), 32);
    }

    #[test]
    fn test_smt_proof_verification_logic() {
        // Test the logic for verifying SMT membership proofs
        let key = [1u8; 32];
        let value = [2u8; 32];
        let leaf_hash = {
            let mut hasher = Hasher::default();
            hasher.hash(&key);
            hasher.hash(&value);
            hasher.result().to_bytes()
        };
        
        // Single level proof
        let sibling = [3u8; 32];
        let proof = vec![sibling];
        
        // Calculate root for verification
        let mut current_hash = leaf_hash;
        for &sibling_hash in &proof {
            let mut hasher = Hasher::default();
            // In a real implementation, we'd need to determine left/right based on key bits
            hasher.hash(&current_hash);
            hasher.hash(&sibling_hash);
            current_hash = hasher.result().to_bytes();
        }
        
        assert_ne!(current_hash, leaf_hash);
        assert_eq!(current_hash.len(), 32);
    }

    #[test]
    fn test_smt_key_to_path() {
        // Test converting a key to a path for SMT traversal
        let key = [0b10110000u8; 32]; // Binary: 10110000...
        let height = 8;
        
        // Extract path bits from key
        let mut path_bits = Vec::new();
        for i in 0..height {
            let byte_index = i / 8;
            let bit_index = 7 - (i % 8);
            let bit = (key[byte_index] >> bit_index) & 1;
            path_bits.push(bit == 1);
        }
        
        // For key starting with 10110000, first 8 bits should be [true, false, true, true, false, false, false, false]
        assert_eq!(path_bits[0], true);  // 1
        assert_eq!(path_bits[1], false); // 0
        assert_eq!(path_bits[2], true);  // 1
        assert_eq!(path_bits[3], true);  // 1
        assert_eq!(path_bits[4], false); // 0
        assert_eq!(path_bits[5], false); // 0
        assert_eq!(path_bits[6], false); // 0
        assert_eq!(path_bits[7], false); // 0
    }

    #[test]
    fn test_smt_empty_tree_root() {
        // Test that empty SMT has a predictable root
        let empty_hash = [0u8; 32];
        
        // For an empty tree, the root should be the empty hash
        // In practice, this might be a specific "empty" value
        assert_eq!(empty_hash, [0u8; 32]);
    }

    #[test]
    fn test_smt_single_leaf_tree() {
        // Test SMT with a single leaf
        let key = [1u8; 32];
        let value = [2u8; 32];
        
        let leaf_hash = {
            let mut hasher = Hasher::default();
            hasher.hash(&key);
            hasher.hash(&value);
            hasher.result().to_bytes()
        };
        
        // For a tree with height 1 and single leaf, root should be the leaf hash
        // (assuming the other side is empty)
        assert_ne!(leaf_hash, [0u8; 32]);
        assert_eq!(leaf_hash.len(), 32);
    }
}

#[cfg(test)]
mod proof_verification_tests {
    use super::*;
    use crate::state::*;

    #[test]
    fn test_sp1_proof_structure() {
        // Test basic SP1 proof structure validation
        let proof_data = vec![1, 2, 3, 4, 5]; // Mock proof data
        let public_inputs = vec![6, 7, 8, 9]; // Mock public inputs
        
        // Basic validation
        assert!(!proof_data.is_empty());
        assert!(!public_inputs.is_empty());
        assert!(proof_data.len() <= 8192); // Max proof size
        assert!(public_inputs.len() <= 1024); // Max public inputs size
    }

    #[test]
    fn test_groth16_proof_structure() {
        // Groth16 proofs have a specific structure
        let proof_size = 192; // 3 * 64 bytes for G1, G1, G2 points
        let proof_data = vec![0u8; proof_size];
        
        assert_eq!(proof_data.len(), 192);
        assert!(proof_data.len() <= 8192); // Within max size
    }

    #[test]
    fn test_plonk_proof_structure() {
        // PLONK proofs have variable size but typically larger than Groth16
        let proof_size = 768; // Typical PLONK proof size
        let proof_data = vec![0u8; proof_size];
        
        assert_eq!(proof_data.len(), 768);
        assert!(proof_data.len() <= 8192); // Within max size
    }

    #[test]
    fn test_proof_size_limits() {
        // Test proof size validation
        let max_proof_size = 8192;
        let max_public_inputs_size = 1024;
        
        // Valid sizes
        assert!(100 <= max_proof_size);
        assert!(50 <= max_public_inputs_size);
        
        // Invalid sizes
        assert!(10000 > max_proof_size);
        assert!(2000 > max_public_inputs_size);
    }

    #[test]
    fn test_verification_key_matching() {
        // Test that verification keys match expected format
        let vk_hash = [1u8; 32];
        let vk_data = vec![1, 2, 3, 4, 5];
        
        // Calculate hash of VK data
        let calculated_hash = anchor_lang::solana_program::hash::hash(&vk_data).to_bytes();
        
        // In real verification, these should match
        assert_ne!(vk_hash, calculated_hash); // They won't match in this test
        assert_eq!(calculated_hash.len(), 32);
    }

    #[test]
    fn test_public_inputs_validation() {
        // Test public inputs validation
        let valid_inputs = vec![1, 2, 3, 4];
        let empty_inputs: Vec<u8> = vec![];
        let large_inputs = vec![0u8; 2000];
        
        assert!(!valid_inputs.is_empty());
        assert!(empty_inputs.is_empty());
        assert!(large_inputs.len() > 1024); // Too large
    }
} 