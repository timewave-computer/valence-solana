// Diff module - state differencing and optimization

pub mod state;
pub mod diff_calculator;
pub mod instructions;
pub mod optimizer;

// Public re-exports
pub use state::*;
pub use diff_calculator::*;
pub use instructions::*;
pub use optimizer::*; 