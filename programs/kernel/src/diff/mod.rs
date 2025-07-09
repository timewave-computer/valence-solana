// Valence Protocol Diff - State difference calculation and atomic processing
// This module provides algorithms for calculating and processing state differences
//
// All diff-related functionality has been consolidated here from the former
// optimization module. This includes:
// - Diff types and data structures
// - Atomic diff processing
// - Batch optimization
// - Performance optimization algorithms



pub mod instructions;
pub mod state;
pub mod diff_calculator;
pub mod atomic_processor;
pub mod batch_optimizer;
pub mod types;
pub mod performance;

// Re-export specific items to avoid ambiguous glob re-exports
pub use instructions::{Initialize, CalculateDiff, ProcessDiffs, VerifyDiffIntegrity, DiffOperation as InstructionsDiffOperation};
pub use state::*;
pub use diff_calculator::*;
pub use atomic_processor::*;
pub use batch_optimizer::*;
pub use types::DiffOperation as TypesDiffOperation;
pub use performance::*; 