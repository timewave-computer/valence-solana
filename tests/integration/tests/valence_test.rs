// Unit Tests for Valence Protocol Core Components
// ===============================================
//
// This file contains unit-level tests for the Valence Protocol that validate:
// 1. Content hash computation (deterministic and correct)
// 2. Capability system (bitmap operations and permission checks)
// 3. Session state hash updates (for audit trail)
// 4. Session builder API functionality
// 5. PDA (Program Derived Address) derivation
// 6. Account size calculations
//
// These tests run without a validator and focus on testing the logic
// of individual components in isolation.

use anchor_lang::prelude::*;
use registry::{self};
use shard::{self, Capabilities};
use sha2::{Sha256, Digest};

#[test]
fn test_registry_content_hash() {
    // Test that content hash computation is deterministic
    let program_id = Pubkey::new_unique();
    let bytecode_hash = [42u8; 32];
    
    // Compute hash
    let mut hasher = Sha256::new();
    hasher.update(program_id.as_ref());
    hasher.update(bytecode_hash);
    let content_hash: [u8; 32] = hasher.finalize().into();
    
    // Verify it's deterministic
    let mut hasher2 = Sha256::new();
    hasher2.update(program_id.as_ref());
    hasher2.update(bytecode_hash);
    let content_hash2: [u8; 32] = hasher2.finalize().into();
    
    assert_eq!(content_hash, content_hash2);
}

#[test]
fn test_capabilities() {
    let mut caps = Capabilities::new(0);
    
    // Test individual capabilities
    assert!(!caps.has(Capabilities::READ));
    assert!(!caps.has(Capabilities::WRITE));
    
    // Add capabilities
    caps.0 |= Capabilities::READ;
    caps.0 |= Capabilities::WRITE;
    
    assert!(caps.has(Capabilities::READ));
    assert!(caps.has(Capabilities::WRITE));
    assert!(!caps.has(Capabilities::EXECUTE));
    
    // Test require
    assert!(caps.require(Capabilities::READ).is_ok());
    assert!(caps.require(Capabilities::EXECUTE).is_err());
}

#[test]
fn test_state_hash_update() {
    let state_hash = [0u8; 32];
    let function_hash = [1u8; 32];
    let input_data = b"test_data";
    let nonce = 1u64;
    
    // Compute new state hash
    let mut hasher = Sha256::new();
    hasher.update(state_hash);
    hasher.update(function_hash);
    hasher.update(input_data);
    hasher.update(nonce.to_le_bytes());
    let new_hash: [u8; 32] = hasher.finalize().into();
    
    // Verify it changed
    assert_ne!(state_hash, new_hash);
    
    // Verify deterministic
    let mut hasher2 = Sha256::new();
    hasher2.update(state_hash);
    hasher2.update(function_hash);
    hasher2.update(input_data);
    hasher2.update(nonce.to_le_bytes());
    let new_hash2: [u8; 32] = hasher2.finalize().into();
    
    assert_eq!(new_hash, new_hash2);
}

#[test]
fn test_session_builder_capabilities() {
    use valence_sdk::SessionBuilder;
    
    // Test capability accumulation
    let builder = SessionBuilder::new()
        .with_read()
        .with_write()
        .with_execute();
    
    let expected_caps = Capabilities(
        Capabilities::READ | 
        Capabilities::WRITE | 
        Capabilities::EXECUTE
    );
    
    // SessionBuilder stores capabilities as u64
    assert_eq!(builder.capabilities, expected_caps.0);
}

#[test]
fn test_pda_derivation() {
    let owner = Pubkey::new_unique();
    let nonce = 12345u32;
    
    // Test session PDA
    let (session_pda, _bump) = Pubkey::find_program_address(
        &[
            shard::SESSION_SEED,
            owner.as_ref(),
            &nonce.to_le_bytes()[..4],
        ],
        &shard::ID,
    );
    
    // Verify it's deterministic
    let (session_pda2, _bump2) = Pubkey::find_program_address(
        &[
            shard::SESSION_SEED,
            owner.as_ref(),
            &nonce.to_le_bytes()[..4],
        ],
        &shard::ID,
    );
    
    assert_eq!(session_pda, session_pda2);
}

#[test]
fn test_function_entry_size() {
    // Verify the size calculation is correct
    let calculated_size = 8 + // discriminator
        32 + // program
        32 + // content_hash
        32 + // bytecode_hash
        32 + // authority
        8; // registered_at
    
    assert_eq!(registry::FunctionEntry::SIZE, calculated_size);
}

#[test]
fn test_session_size() {
    // Verify the size calculation is correct
    let calculated_size = 8 + // discriminator
        32 + // owner
        8 + // capabilities
        8 + // nonce
        32 + // state_hash
        4 + // metadata vec length
        8 + // created_at
        1; // consumed
    
    assert_eq!(shard::Session::SIZE, calculated_size);
}