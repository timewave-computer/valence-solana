// Instructions module for ZK Proof Verifier Program

pub mod initialize;
pub mod register_verification_key;
pub mod verify_proof;
pub mod update_verification_key;
pub mod create_smt;
pub mod update_smt;
pub mod verify_smt_membership;

pub use initialize::*;
pub use register_verification_key::*;
pub use verify_proof::*;
pub use update_verification_key::*;
pub use create_smt::*;
pub use update_smt::*;
pub use verify_smt_membership::*; 