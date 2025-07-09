/// Verification function execution
/// This module handles verification instruction handlers, predicates, and common verification functions
pub mod instructions;
pub mod verifiers;
pub mod state_management;
pub mod registry;
pub mod zk_proof;

// Import error type from the unified error module
pub use crate::error::VerificationError;

// Re-export key types from the correct modules
pub use crate::functions::verification::{VerificationInput, VerificationOutput};

// Re-export verification functions from verifiers module
pub use verifiers::{
    verify_basic_permission, verify_parameter_constraint, verify_system_auth,
    register_basic_verifications, get_basic_permission_function,
    get_parameter_constraint_function, get_system_auth_function,
    verify_block_height, verify_session_creation,
    register_context_verifications, get_block_height_function,
    get_session_creation_function, create_after_height_condition,
    create_before_height_condition, create_between_heights_condition,
    create_session_params,
};

// Re-export state management from state_management module
pub use state_management::{
    PermissionConfig, AuthState, BlockState, ConstraintConfig,
    AuthConfig, ExecutionLevel, ConstraintConfigParams, CallParameters,
    basic_permission_verifier, block_height_verifier, parameter_constraint_verifier,
};

// Re-export registry functionality
pub use registry::{
    DefaultVerificationRegistry, CapabilityConfig, FunctionRegistryState,
    FunctionCategoryStats, FunctionType, RegisteredFunction, FunctionMetadata,
    FunctionPerformance, FunctionDiscoveryIndex,
}; 