//! Valence SDK - Simplified interface for interacting with Valence programs
//! 
//! This SDK provides thin wrappers for:
//! - Gateway routing
//! - Registry operations
//! - Verification proxy interactions
//! - Shard deployment and management
//! - Diff construction helpers

pub mod client;
pub mod gateway;
pub mod registry;
pub mod verification;
pub mod shard;
pub mod diff;
pub mod types;

// Re-export key types
pub use client::ValenceClient;
pub use types::*;

// Prelude for template projects
pub mod prelude {
    pub use anchor_lang::prelude::*;
}

// SDK-specific types can be defined here
#[derive(Debug, Clone)]
pub struct SdkConfig {
    pub rpc_url: String,
    pub commitment: String,
}