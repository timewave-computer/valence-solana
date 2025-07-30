// Core function trait and types
use crate::Environment;
use anchor_lang::prelude::*;

/// Metadata for functions in the shard registry
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct FunctionMetadata {
    pub content_hash: [u8; 32],
    pub name: String,
    pub version: u16,
    pub author: String,
    pub description: String,
    pub dependencies: Vec<[u8; 32]>,
    pub supported_states: Vec<String>,
}

impl FunctionMetadata {
    pub fn new(name: String, version: u16, author: String, description: String) -> Self {
        Self {
            content_hash: [0u8; 32],
            name,
            version,
            author,
            description,
            dependencies: Vec::new(),
            supported_states: Vec::new(),
        }
    }
}

/// Result of function execution
#[derive(Clone, Debug)]
pub enum FunctionResult {
    Success(Option<Vec<u8>>),
    Revert(String),
    OutOfGas,
    Interrupted(String),
}

impl FunctionResult {
    pub fn is_success(&self) -> bool {
        matches!(self, FunctionResult::Success(_))
    }
}

/// Trait for pure functions that deterministically transform state
pub trait PureFunction: Clone {
    type Input;
    type Output;

    fn execute(&self, input: &Self::Input, env: &Environment) -> Result<Self::Output>;

    fn validate_input(&self, _input: &Self::Input) -> Result<()> {
        Ok(())
    }

    fn estimate_compute_units(&self) -> u64 {
        50_000
    }

    fn is_deterministic(&self) -> bool {
        true
    }

    fn metadata(&self) -> Option<FunctionMetadata> {
        None
    }
}