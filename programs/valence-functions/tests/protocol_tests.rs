// Shard system tests for valence-functions


use valence_functions::*;
use valence_functions::escrow::EscrowShard;

#[test]
fn test_shard_trait() {
    let escrow_shard = EscrowShard::default();
    assert_eq!(escrow_shard.name(), "Valence Escrow Shard V1");
    assert_eq!(escrow_shard.version(), 1);
    
    let id = escrow_shard.id();
    assert_eq!(&id[..6], b"ESCROW");
}

#[test]
fn test_shard_upgrade_trait() {
    // Test that the Shard trait methods work
    let shard = EscrowShard::nft_trading();
    assert_eq!(shard.fee_bps, 250);
    assert_eq!(shard.version(), 1);
    
    let p2p_shard = EscrowShard::p2p_trading();
    assert_eq!(p2p_shard.fee_bps, 0);
}

#[test]
fn test_escrow_shard_operations() {
    let shard = EscrowShard::default();
    
    // Test fee calculation
    let fees = shard.calculate_fees(10000).unwrap();
    assert_eq!(fees, 100); // 1% of 10000
    
    // Test expiry validation
    assert!(shard.validate_expiry(3600).is_ok()); // 1 hour
    assert!(shard.validate_trade_amount(1000).is_ok());
    
    // Test expiry calculation
    let created_at = 1234567890;
    let expires_at = shard.calculate_expiry(created_at, None).unwrap();
    assert_eq!(expires_at, created_at + 24 * 3600);
} 