//! Registry state - Function registry models

use anchor_lang::prelude::*;

/// Function entry in the registry
#[account]
pub struct FunctionEntry {
    /// Content hash - the only identifier
    pub hash: [u8; 32],
    /// Program that implements this function
    pub program: Pubkey,
    /// Authority that registered this function
    pub authority: Pubkey,
    /// Required capabilities for this function
    pub required_capabilities: Vec<String>,
}