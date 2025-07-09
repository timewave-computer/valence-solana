/// Capability definitions with namespace scoping
/// This module handles capability account structures, instruction handlers, and namespace access logic
pub mod scoping;
pub mod instructions;
pub mod eval_rules;
pub mod ordering_rules;
pub mod execution_config;

// Re-export main types and functions - be specific to avoid ambiguity
pub use scoping::{
    ValidateNamespaceScope, CheckObjectAccess, VerifyCapabilityComposition,
    CapabilityValidationResult, CapabilityDefinition, CapabilityScope, CapabilityType,
    NamespaceManager, NamespaceAccess,
};

// Import error types from the unified error module
pub use crate::error::{CapabilityError, NamespaceScopingError, ExecutionConfigError};

// Re-export eval-related types
pub use eval_rules::{ShardState, EvalConfig};

// Import execution types from processor singleton
pub use crate::processor::{ExecutionContext, ExecutionResult, ContextBuilder};

// Re-export ordering-related types
pub use ordering_rules::{
    OrderingConstraint, PartialOrder, OrderingRule, FifoOrderingRule,
    PriorityOrderingRule, DependencyOrderingRule, OrderingRuleRegistry,
};

// Re-export execution config types
pub use execution_config::{
    ExecutionConfig, ExecutionMode, ResourceLimits, CompleteExecutionConfig,
}; 