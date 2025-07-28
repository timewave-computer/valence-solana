// Basic integration tests for valence-functions

use anchor_lang::prelude::*;
use valence_functions::states::{EscrowState, EscrowStatus};
use valence_functions::guard_core::GuardFunction;
use valence_functions::*;

// Test helpers
fn mock_pubkey() -> Pubkey {
    Pubkey::new_unique()
}

fn mock_env() -> Environment {
    Environment {
        caller: mock_pubkey(),
        timestamp: 1234567890,
        slot: 100,
        recent_blockhash: [1u8; 32],
    }
}

fn mock_clock() -> Clock {
    Clock {
        slot: 100,
        epoch_start_timestamp: 0,
        epoch: 0,
        leader_schedule_epoch: 0,
        unix_timestamp: 1234567890,
    }
}

#[test]
fn test_escrow_shard_creation() {
    let shard = valence_functions::escrow::p2p_escrow();

    // Basic assertions to ensure shard is created
    assert_eq!(shard.version(), 1);
    assert_eq!(shard.name(), "Valence Escrow Shard V1");
}

#[test]
fn test_environment_struct() {
    let env = mock_env();
    assert_eq!(env.timestamp, 1234567890);
    assert_eq!(env.slot, 100);
}

#[test]
fn test_escrow_state() {
    let seller = mock_pubkey();
    let buyer = mock_pubkey();
    let asset_mint = mock_pubkey();

    let state = EscrowState {
        seller,
        buyer: Some(buyer),
        asset_mint,
        price: 100_000,
        created_at: 1234567890,
        expires_at: 1234567890 + 3600,
        status: EscrowStatus::Committed,
        _reserved: [0u8; 32],
    };

    // Test is_ready_for_completion
    let clock = mock_clock();
    assert!(state.is_ready_for_completion(&clock));

    // Test expiration
    let expired_clock = Clock {
        slot: 100,
        epoch_start_timestamp: 0,
        epoch: 0,
        leader_schedule_epoch: 0,
        unix_timestamp: 1234567890 + 7200,
    };
    assert!(state.is_expired(&expired_clock));
}

#[test]
fn test_escrow_active_guard() {
    let guard = EscrowActiveGuard;

    let state = EscrowState {
        seller: mock_pubkey(),
        buyer: None,
        asset_mint: mock_pubkey(),
        price: 100_000,
        created_at: 1234567890,
        expires_at: 1234567890 + 3600,
        status: EscrowStatus::Open,
        _reserved: [0u8; 32],
    };

    // Test active escrow
    let env = Environment {
        caller: mock_pubkey(),
        timestamp: 1234567890 + 1800, // Halfway through
        slot: 100,
        recent_blockhash: [0u8; 32],
    };
    assert!(guard.check(&state, &[], &env).unwrap());

    // Test expired escrow
    let expired_env = Environment {
        caller: mock_pubkey(),
        timestamp: 1234567890 + 7200, // Past expiration
        slot: 100,
        recent_blockhash: [0u8; 32],
    };
    assert!(!guard.check(&state, &[], &expired_env).unwrap());
}

#[test]
fn test_escrow_party_guard() {
    let seller = mock_pubkey();
    let buyer = mock_pubkey();
    let other = mock_pubkey();

    let guard = EscrowPartyGuard;

    let state = EscrowState {
        seller,
        buyer: Some(buyer),
        asset_mint: mock_pubkey(),
        price: 100_000,
        created_at: 1234567890,
        expires_at: 1234567890 + 3600,
        status: EscrowStatus::Committed,
        _reserved: [0u8; 32],
    };

    // Test seller access
    let seller_env = Environment {
        caller: seller,
        timestamp: 1234567890,
        slot: 100,
        recent_blockhash: [0u8; 32],
    };
    assert!(guard.check(&state, &[], &seller_env).unwrap());

    // Test buyer access
    let buyer_env = Environment {
        caller: buyer,
        timestamp: 1234567890,
        slot: 100,
        recent_blockhash: [0u8; 32],
    };
    assert!(guard.check(&state, &[], &buyer_env).unwrap());

    // Test unauthorized access
    let other_env = Environment {
        caller: other,
        timestamp: 1234567890,
        slot: 100,
        recent_blockhash: [0u8; 32],
    };
    assert!(!guard.check(&state, &[], &other_env).unwrap());
}

#[test]
fn test_escrow_transition_guard() {
    let seller = mock_pubkey();
    let buyer = mock_pubkey();
    let guard = EscrowTransitionGuard;

    // State with no buyer yet
    let no_buyer_state = EscrowState {
        seller,
        buyer: None,
        asset_mint: mock_pubkey(),
        price: 100_000,
        created_at: 1234567890,
        expires_at: 1234567890 + 3600,
        status: EscrowStatus::Open,
        _reserved: [0u8; 32],
    };

    // Test accept offer (operation 0)
    let accept_op = vec![0];
    let buyer_env = Environment {
        caller: buyer,
        timestamp: 1234567890,
        slot: 100,
        recent_blockhash: [0u8; 32],
    };
    assert!(guard
        .check(&no_buyer_state, &accept_op, &buyer_env)
        .unwrap());

    // Seller cannot accept their own offer
    let seller_env = Environment {
        caller: seller,
        timestamp: 1234567890,
        slot: 100,
        recent_blockhash: [0u8; 32],
    };
    assert!(!guard
        .check(&no_buyer_state, &accept_op, &seller_env)
        .unwrap());
}

#[test]
fn test_composite_guards() {
    let guard1 = EscrowActiveGuard;
    let guard2 = EscrowPartyGuard;

    let and_guard = AndGuard {
        guard1: guard1.clone(),
        guard2: guard2.clone(),
    };

    let seller = mock_pubkey();
    let state = EscrowState {
        seller,
        buyer: None,
        asset_mint: mock_pubkey(),
        price: 100_000,
        created_at: 1234567890,
        expires_at: 1234567890 + 3600,
        status: EscrowStatus::Open,
        _reserved: [0u8; 32],
    };

    // Test with both conditions satisfied (active + is seller)
    let seller_env = Environment {
        caller: seller,
        timestamp: 1234567890 + 1800,
        slot: 100,
        recent_blockhash: [0u8; 32],
    };
    assert!(and_guard.check(&state, &[], &seller_env).unwrap());

    // Test with one condition failing (active but not party)
    let other_env = Environment {
        caller: mock_pubkey(),
        timestamp: 1234567890 + 1800,
        slot: 100,
        recent_blockhash: [0u8; 32],
    };
    assert!(!and_guard.check(&state, &[], &other_env).unwrap());
}
