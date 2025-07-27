// State system tests for valence-functions

use anchor_lang::prelude::*;
use valence_functions::*;
use valence_functions::states::{EscrowState, EscrowStatus, EscrowBuilder, StateValidator};



fn mock_clock() -> Clock {
    Clock {
        slot: 100,
        epoch_start_timestamp: 1000,
        epoch: 1,
        leader_schedule_epoch: 1,
        unix_timestamp: 1234567890,
    }
}

#[test]
fn test_environment_validity() {
    let valid_env = Environment {
        caller: Pubkey::new_unique(),
        timestamp: 1234567890,
        slot: 100,
        recent_blockhash: [0u8; 32],
    };
    assert!(valid_env.is_valid());

    let invalid_env = Environment {
        caller: Pubkey::new_unique(),
        timestamp: 0,
        slot: 0,
        recent_blockhash: [0u8; 32],
    };
    assert!(!invalid_env.is_valid());
}

#[test]
fn test_escrow_state_validation() {
    let valid_escrow = EscrowState {
        seller: Pubkey::new_unique(),
        buyer: None,
        asset_mint: Pubkey::new_unique(),
        price: 1000,
        created_at: 1234567800,
        expires_at: 1234567800 + 3600, // 1 hour duration (minimum allowed)
        status: EscrowStatus::Open,
        _reserved: [0u8; 32],
    };
    
    assert!(valid_escrow.validate().is_ok());
    assert!(valid_escrow.allows_operation("commit"));
    assert!(!valid_escrow.allows_operation("complete"));
}

#[test] 
fn test_escrow_builder() {
    let clock = mock_clock();

    let escrow = EscrowBuilder::new()
        .seller(Pubkey::new_unique())
        .asset_mint(Pubkey::new_unique())
        .price(1000)
        .duration(3600)
        .build(&clock)
        .unwrap();

    assert_eq!(escrow.price, 1000);
    assert_eq!(escrow.status, EscrowStatus::Open);
    assert!(escrow.buyer.is_none());
} 