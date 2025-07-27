// State machine guard implementations
use super::core::GuardFunction;
use crate::Environment;
use anchor_lang::prelude::*;
use std::collections::HashMap;

/// Linear state transition guard
#[derive(Clone, Debug)]
pub struct LinearFlowGuard {
    pub expected_sequence: Vec<u8>,
    pub current_position: usize,
}

impl LinearFlowGuard {
    pub fn new(expected_sequence: Vec<u8>) -> Self {
        Self {
            expected_sequence,
            current_position: 0,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.current_position >= self.expected_sequence.len()
    }
}

impl GuardFunction for LinearFlowGuard {
    type State = ();

    fn check(&self, _state: &Self::State, operation: &[u8], _env: &Environment) -> Result<bool> {
        if operation.is_empty() || self.is_complete() {
            return Ok(false);
        }
        Ok(operation[0] == self.expected_sequence[self.current_position])
    }

    fn description(&self) -> &'static str {
        "Enforces linear operation sequence"
    }

    fn compute_cost(&self) -> u64 {
        500
    }
}

/// Finite state machine guard
#[derive(Clone, Debug)]
pub struct StateMachineGuard {
    pub current_state: u8,
    pub transitions: HashMap<(u8, u8), u8>,
    pub active_states: Vec<u8>,
}

impl StateMachineGuard {
    pub fn new(initial_state: u8) -> Self {
        Self {
            current_state: initial_state,
            transitions: HashMap::new(),
            active_states: Vec::new(),
        }
    }

    pub fn add_transition(&mut self, from_state: u8, operation: u8, to_state: u8) {
        self.transitions.insert((from_state, operation), to_state);
    }

    pub fn add_active_state(&mut self, state: u8) {
        if !self.active_states.contains(&state) {
            self.active_states.push(state);
        }
    }

    pub fn is_active(&self) -> bool {
        self.active_states.contains(&self.current_state)
    }
}

impl GuardFunction for StateMachineGuard {
    type State = ();

    fn check(&self, _state: &Self::State, operation: &[u8], _env: &Environment) -> Result<bool> {
        if operation.is_empty() {
            return Ok(false);
        }

        let operation_type = operation[0];
        let valid_transition = self
            .transitions
            .contains_key(&(self.current_state, operation_type));
        let state_allows = self.is_active();

        Ok(valid_transition && state_allows)
    }

    fn description(&self) -> &'static str {
        "Finite state machine guard"
    }

    fn compute_cost(&self) -> u64 {
        800
    }
}