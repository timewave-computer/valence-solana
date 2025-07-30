// Function composition utilities
use super::core::{FunctionMetadata, PureFunction};
use crate::Environment;
use anchor_lang::prelude::*;

/// Compose two functions: f(g(x))
#[derive(Clone, Debug)]
pub struct ComposedFunction<F, G> {
    pub inner: G,
    pub outer: F,
}

impl<F, G> ComposedFunction<F, G> {
    pub fn new(inner: G, outer: F) -> Self {
        Self { inner, outer }
    }
}

impl<F, G, T> PureFunction for ComposedFunction<F, G>
where
    G: PureFunction<Output = T>,
    F: PureFunction<Input = T>,
{
    type Input = G::Input;
    type Output = F::Output;

    fn execute(&self, input: &Self::Input, env: &Environment) -> Result<Self::Output> {
        let intermediate = self.inner.execute(input, env)?;
        self.outer.execute(&intermediate, env)
    }

    fn validate_input(&self, input: &Self::Input) -> Result<()> {
        self.inner.validate_input(input)
    }

    fn estimate_compute_units(&self) -> u64 {
        self.inner.estimate_compute_units() + self.outer.estimate_compute_units() + 1_000
    }

    fn is_deterministic(&self) -> bool {
        self.inner.is_deterministic() && self.outer.is_deterministic()
    }
}

/// Conditional function implementing if-then-else logic
#[derive(Clone)]
pub struct ConditionalFunction<P, T, E> {
    pub predicate: P,
    pub then_branch: T,
    pub else_branch: E,
}

impl<P, T, E> ConditionalFunction<P, T, E> {
    pub fn new(predicate: P, then_branch: T, else_branch: E) -> Self {
        Self {
            predicate,
            then_branch,
            else_branch,
        }
    }
}

impl<P, T, E, Input, Output> PureFunction for ConditionalFunction<P, T, E>
where
    Input: Clone,
    P: Fn(&Input, &Environment) -> Result<bool> + Clone,
    T: PureFunction<Input = Input, Output = Output>,
    E: PureFunction<Input = Input, Output = Output>,
{
    type Input = Input;
    type Output = Output;

    fn execute(&self, input: &Self::Input, env: &Environment) -> Result<Self::Output> {
        if (self.predicate)(input, env)? {
            self.then_branch.execute(input, env)
        } else {
            self.else_branch.execute(input, env)
        }
    }

    fn validate_input(&self, input: &Self::Input) -> Result<()> {
        self.then_branch.validate_input(input)?;
        self.else_branch.validate_input(input)?;
        Ok(())
    }

    fn estimate_compute_units(&self) -> u64 {
        let then_cost = self.then_branch.estimate_compute_units();
        let else_cost = self.else_branch.estimate_compute_units();
        then_cost.max(else_cost) + 5_000
    }

    fn is_deterministic(&self) -> bool {
        self.then_branch.is_deterministic() && self.else_branch.is_deterministic()
    }
}

/// Wrapper for versioned functions
#[derive(Clone, Debug)]
pub struct VersionedFunction<F> {
    pub version: u16,
    pub function: F,
    pub deprecated: bool,
    pub upgrade_to: Option<[u8; 32]>,
}

impl<F> VersionedFunction<F> {
    pub fn new(version: u16, function: F) -> Self {
        Self {
            version,
            function,
            deprecated: false,
            upgrade_to: None,
        }
    }
}

impl<F: PureFunction> PureFunction for VersionedFunction<F> {
    type Input = F::Input;
    type Output = F::Output;

    fn execute(&self, input: &Self::Input, env: &Environment) -> Result<Self::Output> {
        if self.deprecated {
            msg!("Warning: Function version {} is deprecated", self.version);
        }
        self.function.execute(input, env)
    }

    fn validate_input(&self, input: &Self::Input) -> Result<()> {
        self.function.validate_input(input)
    }

    fn estimate_compute_units(&self) -> u64 {
        self.function.estimate_compute_units()
    }

    fn is_deterministic(&self) -> bool {
        self.function.is_deterministic()
    }

    fn metadata(&self) -> Option<FunctionMetadata> {
        let mut metadata = self.function.metadata()?;
        metadata.version = self.version;
        Some(metadata)
    }
}