//! Shard deployment and interaction helpers

use anchor_lang::prelude::*;

/// Deploy a new shard instance
pub fn deploy_shard() -> Result<Pubkey> {
    todo!("Implement shard deployment")
}

/// Build session request for a shard
pub fn build_session_request(
    _shard: Pubkey,
    _capabilities: Vec<String>,
    _init_state_hash: [u8; 32],
) -> Result<()> {
    todo!("Implement session request")
}

/// Build bundle execution instruction
pub fn build_bundle_execution(
    _shard: Pubkey,
    _operations: Vec<Operation>,
) -> Result<()> {
    todo!("Implement bundle execution")
}

#[derive(Debug, Clone)]
pub struct Operation {
    pub function_hash: [u8; 32],
    pub args: Vec<u8>,
    pub expected_diff: Option<[u8; 32]>,
}