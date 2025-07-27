// Guard composition utilities for building complex authorization patterns
use crate::{guard_core::GuardFunction, Environment};
use anchor_lang::prelude::*;

/// Combine two guards with AND logic - both must pass
#[derive(Clone, Debug)]
pub struct AndGuard<G1, G2> {
    pub guard1: G1,
    pub guard2: G2,
}

impl<G1, G2> AndGuard<G1, G2> {
    pub fn new(guard1: G1, guard2: G2) -> Self {
        Self { guard1, guard2 }
    }
}

/// Combine two guards with OR logic - either can pass
#[derive(Clone, Debug)]
pub struct OrGuard<G1, G2> {
    pub guard1: G1,
    pub guard2: G2,
}

impl<G1, G2> OrGuard<G1, G2> {
    pub fn new(guard1: G1, guard2: G2) -> Self {
        Self { guard1, guard2 }
    }
}

/// Invert a guard's result (NOT logic)
#[derive(Clone, Debug)]
pub struct NotGuard<G> {
    pub guard: G,
}

impl<G> NotGuard<G> {
    pub fn new(guard: G) -> Self {
        Self { guard }
    }
}

/// Helper to create AND guard
pub fn and<G1, G2>(guard1: G1, guard2: G2) -> AndGuard<G1, G2> {
    AndGuard::new(guard1, guard2)
}

/// Helper to create OR guard
pub fn or<G1, G2>(guard1: G1, guard2: G2) -> OrGuard<G1, G2> {
    OrGuard::new(guard1, guard2)
}

/// Helper to create NOT guard
pub fn not<G>(guard: G) -> NotGuard<G> {
    NotGuard::new(guard)
}

/// Conditional guard that applies different logic based on condition
pub struct ConditionalGuard<S, G, F>
where
    F: Fn(&S) -> bool,
    G: Fn(&S) -> bool,
{
    pub condition: F,
    pub then_guard: G,
    _phantom: std::marker::PhantomData<S>,
}

impl<S, G, F> ConditionalGuard<S, G, F>
where
    F: Fn(&S) -> bool,
    G: Fn(&S) -> bool,
{
    pub fn new(condition: F, then_guard: G) -> Self {
        Self {
            condition,
            then_guard,
            _phantom: std::marker::PhantomData,
        }
    }
}

// ================================
// GuardFunction implementations
// ================================

impl<G1, G2, S> GuardFunction for AndGuard<G1, G2>
where
    G1: GuardFunction<State = S>,
    G2: GuardFunction<State = S>,
{
    type State = S;

    fn check(&self, state: &Self::State, operation: &[u8], env: &Environment) -> Result<bool> {
        Ok(self.guard1.check(state, operation, env)? && self.guard2.check(state, operation, env)?)
    }

    fn description(&self) -> &'static str {
        "Combined AND guard"
    }

    fn compute_cost(&self) -> u64 {
        self.guard1.compute_cost() + self.guard2.compute_cost() + 100
    }
}

impl<G1, G2, S> GuardFunction for OrGuard<G1, G2>
where
    G1: GuardFunction<State = S>,
    G2: GuardFunction<State = S>,
{
    type State = S;

    fn check(&self, state: &Self::State, operation: &[u8], env: &Environment) -> Result<bool> {
        Ok(self.guard1.check(state, operation, env)? || self.guard2.check(state, operation, env)?)
    }

    fn description(&self) -> &'static str {
        "Combined OR guard"
    }

    fn compute_cost(&self) -> u64 {
        self.guard1.compute_cost() + self.guard2.compute_cost() + 100
    }
}

impl<G, S> GuardFunction for NotGuard<G>
where
    G: GuardFunction<State = S>,
{
    type State = S;

    fn check(&self, state: &Self::State, operation: &[u8], env: &Environment) -> Result<bool> {
        Ok(!self.guard.check(state, operation, env)?)
    }

    fn description(&self) -> &'static str {
        "Inverted guard"
    }

    fn compute_cost(&self) -> u64 {
        self.guard.compute_cost() + 50
    }
}