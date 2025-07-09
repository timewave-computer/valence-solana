/// Function registry and execution for the Valence Protocol
/// This module provides comprehensive function management capabilities
pub mod instructions;
pub mod verification;
pub mod registry;
pub mod execution;
pub mod metadata;
pub mod types;

// Re-export key types and functions
pub use instructions::{FunctionInput, FunctionOutput};
pub use types::{FunctionResult, PureFunction, FunctionError, EvalContext};
pub use verification::{VerificationFunction, VerificationFunctionTrait, VerificationInput, VerificationOutput};
pub use execution::{FunctionChain, FunctionAggregation, FunctionStep};

// Import error type from the unified error module
pub use crate::error::FunctionCompositionError; 