/// Verification function execution
/// This module handles verification instruction handlers, predicates, and common verification functions

pub mod instructions;
pub mod predicates;
pub mod basic_verifications;
pub mod context_verifications;
pub mod zk_proof;
pub mod common;
pub mod state;
pub mod default_verification_registry;

// Import error type from the unified error module
pub use crate::error::VerificationError;

// Re-export key types from the correct modules
pub use crate::functions::verification::{VerificationInput, VerificationOutput};

// Re-export verification functions from consolidated modules
pub use basic_verifications::{
    verify_basic_permission, verify_parameter_constraint, verify_system_auth,
    register_basic_verifications, get_basic_permission_function,
    get_parameter_constraint_function, get_system_auth_function,
};
pub use context_verifications::{
    verify_block_height, verify_session_creation,
    register_context_verifications, get_block_height_function,
    get_session_creation_function, create_after_height_condition,
    create_before_height_condition, create_between_heights_condition,
    create_session_params,
};
pub use default_verification_registry::DefaultVerificationRegistry; 