// Individual function implementations for valence-functions
// 
// Each module in this directory contains a single function that can be 
// imported and used by shard programs through the function registry.

/// Core function trait and metadata definitions
pub mod core;

/// Function composition utilities
pub mod composition;

/// Identity function - returns input unchanged (for testing)
pub mod identity;

/// Zero-knowledge proof verification function
pub mod zk_verify;

/// Safe mathematical addition function
pub mod math_add;

/// Token account validation function
pub mod token_validate;

// Re-export the functions for easy access
pub use identity::identity;
pub use zk_verify::zk_verify;
pub use math_add::math_add;
pub use token_validate::token_validate;