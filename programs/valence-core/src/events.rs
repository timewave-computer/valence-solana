use anchor_lang::prelude::*;

// ================================
// Protocol Events
// ================================

/// Emitted when a new session account is created
/// Provides audit trail for account creation and configuration
#[event]
pub struct AccountCreated {
    // Account identifiers
    pub account: Pubkey,                   // The newly created account address
    pub session: Pubkey,                   // Parent session managing this account
    
    // Configuration details
    pub protocol_type: [u8; 32],           // Protocol identifier for choreography
    pub verifier: Pubkey,                  // Verifier program responsible for authorization
}