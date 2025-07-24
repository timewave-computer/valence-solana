//! Optional extensions for Valence
//! Enable features as needed: math, events, batching

#[cfg(feature = "math")]
pub mod math;

#[cfg(feature = "events")]  
pub mod events;

#[cfg(feature = "batching")]
pub mod batching;

// Example verifier implementations
pub mod examples;

// Re-export commonly used types when math feature is enabled
#[cfg(feature = "math")]
pub use math::FixedPoint;