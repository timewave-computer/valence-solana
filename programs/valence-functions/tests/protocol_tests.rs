// Protocol system tests for valence-functions


use valence_functions::*;
use valence_functions::escrow::EscrowProtocol;

#[test]
fn test_protocol_trait() {
    let escrow_protocol = EscrowProtocol::default();
    assert_eq!(escrow_protocol.name(), "Valence Escrow Protocol V1");
    assert_eq!(escrow_protocol.version(), 1);
    
    let id = escrow_protocol.id();
    assert_eq!(&id[..6], b"ESCROW");
}

#[test]
fn test_protocol_upgrade_trait() {
    // Test that the Protocol trait methods work
    let protocol = EscrowProtocol::nft_trading();
    assert_eq!(protocol.fee_bps, 250);
    assert_eq!(protocol.version(), 1);
    
    let p2p_protocol = EscrowProtocol::p2p_trading();
    assert_eq!(p2p_protocol.fee_bps, 0);
}

#[test]
fn test_escrow_protocol_operations() {
    let protocol = EscrowProtocol::default();
    
    // Test fee calculation
    let fees = protocol.calculate_fees(10000).unwrap();
    assert_eq!(fees, 100); // 1% of 10000
    
    // Test expiry validation
    assert!(protocol.validate_expiry(3600).is_ok()); // 1 hour
    assert!(protocol.validate_trade_amount(1000).is_ok());
    
    // Test expiry calculation
    let created_at = 1234567890;
    let expires_at = protocol.calculate_expiry(created_at, None).unwrap();
    assert_eq!(expires_at, created_at + 24 * 3600);
} 