// Valence SDK - Clean, concise interface for interacting with the Valence protocol

pub mod account_abstraction;
pub mod client;
pub mod error;
pub mod session;
pub mod atomic;
pub mod guards;
pub mod compute;

pub use account_abstraction::*;
pub use client::*;
pub use error::*;
pub use session::*;
pub use atomic::*;
pub use guards::*;

// Re-export commonly used types
pub use anchor_client::{Client, Cluster};
pub use anchor_lang::prelude::*;
pub use solana_sdk::{
    signature::{Keypair, Signature},
    signer::Signer,
    transaction::Transaction,
};

// Re-export valence types
pub use valence_core::{
    guards::{Guard, CompiledGuard, GuardOp, CPIManifestEntry},
    state::{Session, SessionScope, CreateSessionParams, SessionSharedData},
    operations::{SessionOperation, OperationBatch},
};

pub type Result<T> = std::result::Result<T, error::SdkError>;