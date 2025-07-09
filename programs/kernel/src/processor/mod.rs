// Valence Protocol Processor - Stateless execution shared functionality
// This module provides common execution machinery for capability processing



pub mod instructions;
pub mod state;
pub mod execution_engine;
pub mod verification_orchestrator;
pub mod context_builder;

pub use instructions::*;
pub use state::*;
pub use execution_engine::*;
pub use verification_orchestrator::*;
pub use context_builder::*; 