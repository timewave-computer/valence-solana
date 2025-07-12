//! Gateway errors

use anchor_lang::prelude::*;

#[error_code]
pub enum GatewayError {
    #[msg("Gateway is paused")]
    GatewayPaused,
    
    #[msg("Invalid routing target")]
    InvalidTarget,
    
    #[msg("Unauthorized access")]
    Unauthorized,
    
    #[msg("Invalid instruction data")]
    InvalidInstructionData,
    
    #[msg("Routing failed")]
    RoutingFailed,
}