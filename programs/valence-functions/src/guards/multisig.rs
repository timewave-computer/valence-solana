// Multi-signature guard implementations
use super::core::GuardFunction;
use crate::Environment;
use anchor_lang::prelude::*;

/// Multi-signature guard requiring multiple approvals
#[derive(Clone, Debug)]
pub struct MultiSigGuard {
    pub required_signatures: u8,
    pub authorized_signers: Vec<Pubkey>,
    pub current_signatures: Vec<Pubkey>,
}

impl MultiSigGuard {
    pub fn new(required_signatures: u8, authorized_signers: Vec<Pubkey>) -> Self {
        Self {
            required_signatures,
            authorized_signers,
            current_signatures: Vec::new(),
        }
    }

    pub fn has_enough_signatures(&self) -> bool {
        self.current_signatures.len() >= self.required_signatures as usize
    }
}

impl GuardFunction for MultiSigGuard {
    type State = ();

    fn check(&self, _state: &Self::State, _operation: &[u8], env: &Environment) -> Result<bool> {
        let caller_authorized = self.authorized_signers.contains(&env.caller);
        let enough_signatures = self.has_enough_signatures();
        Ok(caller_authorized && enough_signatures)
    }

    fn description(&self) -> &'static str {
        "Multi-signature guard"
    }

    fn compute_cost(&self) -> u64 {
        500 + (self.authorized_signers.len() as u64 * 50)
    }
}