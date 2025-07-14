//! Property-based tests for Valence Protocol security invariants
//! 
//! This test suite uses property-based testing to verify critical security
//! properties of the Valence Protocol. Each module tests a specific aspect:
//! 
//! - `session_security`: Session lifecycle and state management properties
//! - `capability_enforcement`: Capability-based access control invariants
//! - `function_registration`: Function registry consistency and determinism
//! - `state_isolation`: Memory and state isolation between components
//! - `resource_limits`: Resource consumption limits and DoS prevention
//! - `authorization`: Authentication and authorization properties

#![cfg(test)]

pub mod session_security;
pub mod capability_enforcement;
pub mod function_registration;
pub mod state_isolation;
pub mod resource_limits;
pub mod authorization;