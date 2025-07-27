// Common function implementations
use super::core::{FunctionMetadata, PureFunction};
use crate::Environment;
use anchor_lang::prelude::*;

/// Identity function that returns input unchanged
#[derive(Clone, Debug)]
pub struct IdentityFunction;

impl PureFunction for IdentityFunction {
    type Input = u64;
    type Output = u64;

    fn execute(&self, input: &Self::Input, _env: &Environment) -> Result<Self::Output> {
        Ok(*input)
    }

    fn estimate_compute_units(&self) -> u64 {
        1_000
    }

    fn metadata(&self) -> Option<FunctionMetadata> {
        Some(FunctionMetadata::new(
            "identity".to_string(),
            1,
            "valence-functions".to_string(),
            "Returns input unchanged".to_string(),
        ))
    }
}