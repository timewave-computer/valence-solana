//! Verifier state - Verifier registry

use anchor_lang::prelude::*;

/// Verifier entry in the registry
#[account]
pub struct VerifierEntry {
    /// Label for this verifier (e.g., "balance_check")
    pub label: String,
    /// Program that implements the verifier
    pub program: Pubkey,
    /// Authority that registered this verifier
    pub authority: Pubkey,
    /// When this verifier was registered
    pub registered_at: i64,
}