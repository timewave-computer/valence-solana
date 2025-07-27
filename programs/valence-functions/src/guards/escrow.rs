// Escrow-specific guard implementations
use super::core::GuardFunction;
use crate::{states::EscrowState, Environment};
use anchor_lang::prelude::*;

/// Ensures escrow has not expired
#[derive(Clone, Debug)]
pub struct EscrowActiveGuard;

impl GuardFunction for EscrowActiveGuard {
    type State = EscrowState;

    fn check(&self, state: &Self::State, _operation: &[u8], env: &Environment) -> Result<bool> {
        Ok(env.timestamp < state.expires_at)
    }

    fn description(&self) -> &'static str {
        "Ensures escrow has not expired"
    }

    fn compute_cost(&self) -> u64 {
        500
    }
}

/// Verifies caller is an authorized party
#[derive(Clone, Debug)]
pub struct EscrowPartyGuard;

impl GuardFunction for EscrowPartyGuard {
    type State = EscrowState;

    fn check(&self, state: &Self::State, _operation: &[u8], env: &Environment) -> Result<bool> {
        Ok(env.caller == state.seller || state.buyer == Some(env.caller))
    }

    fn description(&self) -> &'static str {
        "Verifies caller is seller or committed buyer"
    }

    fn compute_cost(&self) -> u64 {
        300
    }
}

/// Validates escrow state transitions
#[derive(Clone, Debug)]
pub struct EscrowTransitionGuard;

impl GuardFunction for EscrowTransitionGuard {
    type State = EscrowState;

    fn check(&self, state: &Self::State, operation: &[u8], env: &Environment) -> Result<bool> {
        if operation.is_empty() {
            return Ok(false);
        }

        match operation[0] {
            0 => {
                // Accept offer
                Ok(state.buyer.is_none() && env.caller != state.seller)
            }
            1 => {
                // Complete transaction
                Ok(state.buyer.is_some() && state.buyer == Some(env.caller))
            }
            2 => {
                // Cancel escrow
                Ok(env.caller == state.seller && state.buyer.is_none())
            }
            3 => {
                // Cancel after expiry
                Ok(env.caller == state.seller && env.timestamp >= state.expires_at)
            }
            _ => Ok(false),
        }
    }

    fn description(&self) -> &'static str {
        "Validates escrow state transitions"
    }

    fn compute_cost(&self) -> u64 {
        800
    }
}

/// Enforces minimum price requirements
#[derive(Clone, Debug)]
pub struct EscrowMinPriceGuard {
    pub min_price: u64,
}

impl EscrowMinPriceGuard {
    pub fn new(min_price: u64) -> Self {
        Self { min_price }
    }
}

impl GuardFunction for EscrowMinPriceGuard {
    type State = EscrowState;

    fn check(&self, state: &Self::State, _operation: &[u8], _env: &Environment) -> Result<bool> {
        Ok(state.price >= self.min_price)
    }

    fn description(&self) -> &'static str {
        "Enforces minimum price requirements"
    }

    fn is_stateless(&self) -> bool {
        true
    }

    fn compute_cost(&self) -> u64 {
        200
    }
}