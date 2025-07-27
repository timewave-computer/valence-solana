// Core guard trait and types
use crate::Environment;
use anchor_lang::prelude::*;

/// Guard functions provide boolean authorization logic
pub trait GuardFunction: Clone {
    /// The state type this guard operates on
    type State;

    /// Execute the guard check
    fn check(&self, state: &Self::State, operation: &[u8], env: &Environment) -> Result<bool>;

    /// Get a human-readable description
    fn description(&self) -> &'static str {
        "Generic guard"
    }

    /// Check if this guard is stateless
    fn is_stateless(&self) -> bool {
        false
    }

    /// Estimate compute units required
    fn compute_cost(&self) -> u64 {
        1_000
    }
}