// Valence SDK - Clean, concise interface for interacting with the Valence protocol

pub mod client;
pub mod error;
pub mod session;
pub mod compute;
pub mod move_semantics;

pub use client::*;
pub use error::*;
pub use session::*;
pub use move_semantics::*;

// Re-export commonly used types
pub use anchor_client::{Client, Cluster};
pub use anchor_lang::prelude::*;
pub use solana_sdk::{
    signature::{Keypair, Signature},
    signer::Signer,
    transaction::Transaction,
};

// Re-export valence types
pub use valence_kernel::{
    KernelOperation,
    OperationBatch,
    state::CreateSessionParams,
};

pub type Result<T> = std::result::Result<T, error::SdkError>;