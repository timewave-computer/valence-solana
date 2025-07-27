// Guard system tests for valence-functions

use anchor_lang::prelude::*;
use valence_functions::*;
use valence_functions::guard_core::GuardFunction;
use valence_functions::guards::escrow::{EscrowActiveGuard, EscrowPartyGuard};
use valence_functions::guards::time::{TimeWindowGuard, RateLimitGuard};
use valence_functions::guards::state_machine::{LinearFlowGuard, StateMachineGuard};
use valence_functions::guards::multisig::MultiSigGuard;
use valence_functions::utils::guard_composition::{AndGuard, OrGuard, NotGuard};
use valence_functions::states::{EscrowState, EscrowStatus};

#[test]
fn test_escrow_active_guard() {
    let escrow = EscrowState {
        seller: Pubkey::new_unique(),
        buyer: None,
        asset_mint: Pubkey::new_unique(),
        price: 1000,
        created_at: 1234567800,
        expires_at: 1234567900, // In the future
        status: EscrowStatus::Open,
        _reserved: [0u8; 32],
    };

    let env = Environment {
        caller: Pubkey::new_unique(),
        timestamp: 1234567850, // Before expiry
        slot: 100,
        recent_blockhash: [1u8; 32],
    };

    let guard = EscrowActiveGuard;
    let result = guard.check(&escrow, &[], &env).unwrap();
    assert!(result, "Escrow should be active before expiry");

    // Test expired escrow
    let env_expired = Environment {
        caller: Pubkey::new_unique(),
        timestamp: 1234567950, // After expiry
        slot: 100,
        recent_blockhash: [2u8; 32],
    };

    let result_expired = guard.check(&escrow, &[], &env_expired).unwrap();
    assert!(!result_expired, "Escrow should be inactive after expiry");
}

#[test]
fn test_guard_composition() {
    let active_guard = EscrowActiveGuard;
    let party_guard = EscrowPartyGuard;
    
    let and_guard = AndGuard::new(active_guard.clone(), party_guard.clone());
    let or_guard = OrGuard::new(active_guard.clone(), party_guard.clone());
    let not_guard = NotGuard::new(active_guard.clone());
    
    // These guards should compile and be cloneable
    let _cloned_and = and_guard.clone();
    let _cloned_or = or_guard.clone();
    let _cloned_not = not_guard.clone();
}

#[test]
fn test_time_based_guards() {
    let time_guard = TimeWindowGuard::new(1234567800, 1234567900);
    let rate_guard = RateLimitGuard::new(10, 3600);
    
    let env = Environment {
        caller: Pubkey::new_unique(),
        timestamp: 1234567850,
        slot: 100,
        recent_blockhash: [0u8; 32],
    };
    
    // Test within time window
    let result = time_guard.check(&(), &[], &env).unwrap();
    assert!(result, "Should be within time window");
    
    // Test rate limiting
    let result = rate_guard.check(&(), &[], &env).unwrap();
    assert!(result, "Should not exceed rate limit initially");
}

#[test]
fn test_state_machine_guards() {
    let linear_guard = LinearFlowGuard::new(vec![1, 2, 3]);
    assert!(!linear_guard.is_complete());
    
    let mut state_machine = StateMachineGuard::new(0);
    state_machine.add_transition(0, 1, 1);
    state_machine.add_active_state(0);
    assert!(state_machine.is_active());
}

#[test]
fn test_multisig_guard() {
    let signers = vec![Pubkey::new_unique(), Pubkey::new_unique()];
    let multisig = MultiSigGuard::new(2, signers);
    assert!(!multisig.has_enough_signatures());
} 