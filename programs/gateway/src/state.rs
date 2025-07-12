//! Gateway state - Minimal routing state

use anchor_lang::prelude::*;

/// Gateway configuration (if needed for routing rules)
#[account]
pub struct GatewayConfig {
    /// Authority that can update routing rules
    pub authority: Pubkey,
    /// Whether gateway is paused
    pub is_paused: bool,
    /// Reserved space for future use
    pub _reserved: [u8; 32],
}