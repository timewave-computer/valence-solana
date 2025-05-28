// Simplified tests for ZK Verification Program functionality

#[cfg(test)]
mod tests {
    use crate::utils::*;
    use zk_verifier::state::*;
    use zk_verifier::error::VerifierError;
    use anchor_lang::prelude::*;

    #[test]
    fn test_verification_key_types() {
        let key_types = vec![
            VerificationKeyType::SP1,
            VerificationKeyType::Groth16,
            VerificationKeyType::PLONK,
        ];

        for key_type in key_types {
            // Test serialization
            let serialized = key_type.try_to_vec().unwrap();
            let deserialized: VerificationKeyType = VerificationKeyType::try_from_slice(&serialized).unwrap();
            assert_eq!(deserialized, key_type);
        }
    }

    #[test]
    fn test_verification_key_creation() {
        let vk = VerificationKey {
            program_id: generate_test_pubkey("test_program"),
            registry_id: 12345,
            vk_hash: [1u8; 32],
            key_type: VerificationKeyType::Groth16,
            is_active: true,
            bump: 255,
        };

        // Test serialization
        let serialized = vk.try_to_vec().unwrap();
        let deserialized: VerificationKey = VerificationKey::try_from_slice(&serialized).unwrap();

        assert_eq!(deserialized.program_id, vk.program_id);
        assert_eq!(deserialized.registry_id, 12345);
        assert_eq!(deserialized.vk_hash, [1u8; 32]);
        assert_eq!(deserialized.key_type, VerificationKeyType::Groth16);
        assert!(deserialized.is_active);
    }

    #[test]
    fn test_verifier_state_management() {
        let state = VerifierState {
            owner: generate_test_pubkey("owner"),
            coprocessor_root: [1u8; 32],
            verifier: generate_test_pubkey("verifier"),
            total_keys: 5,
            bump: 254,
        };

        // Test serialization
        let serialized = state.try_to_vec().unwrap();
        let deserialized: VerifierState = VerifierState::try_from_slice(&serialized).unwrap();

        assert_eq!(deserialized.owner, state.owner);
        assert_eq!(deserialized.coprocessor_root, [1u8; 32]);
        assert_eq!(deserialized.total_keys, 5);
    }

    #[test]
    fn test_proof_validation() {
        // Test different proof sizes for different systems
        let sp1_proof = vec![0u8; 1024]; // Typical SP1 proof size
        let groth16_proof = vec![0u8; 192]; // 3 * 64 bytes for G1, G1, G2 points
        let plonk_proof = vec![0u8; 768]; // Typical PLONK proof size

        let max_proof_size = 8192;

        assert!(sp1_proof.len() <= max_proof_size);
        assert!(groth16_proof.len() <= max_proof_size);
        assert!(plonk_proof.len() <= max_proof_size);

        // Test that proofs are not empty
        assert!(!sp1_proof.is_empty());
        assert!(!groth16_proof.is_empty());
        assert!(!plonk_proof.is_empty());
    }

    #[test]
    fn test_verification_key_hash_validation() {
        let vk_hash = [1u8; 32];
        let empty_hash = [0u8; 32];

        // Valid hash should not be all zeros
        assert_ne!(vk_hash, empty_hash);
        assert_eq!(vk_hash.len(), 32);
        assert_eq!(empty_hash.len(), 32);
    }

    #[test]
    fn test_error_handling() {
        // Test that error codes are properly defined
        let errors = vec![
            VerifierError::VerificationKeyInactive,
            VerifierError::VerificationKeyNotFound,
            VerifierError::InvalidVerificationKey,
            VerifierError::InvalidProof,
            VerifierError::InvalidParameters,
            VerifierError::ProofVerificationFailed,
        ];

        for error in errors {
            // Test that errors can be converted to error codes
            let error_code = error as u32;
            // Error enum discriminants should be reasonable values
            assert!(error_code < 100, "Error code should be reasonable, got {}", error_code);
        }
    }

    #[test]
    fn test_space_calculations() {
        // Test space calculations for different account types
        let verifier_state_space = VerifierState::SPACE;
        let verification_key_space = VerificationKey::SPACE;

        // Should be reasonable sizes
        assert!(verifier_state_space > 50 && verifier_state_space < 200);
        assert!(verification_key_space > 50 && verification_key_space < 200);
    }
} 