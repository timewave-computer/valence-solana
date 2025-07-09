/// Shard instructions module


/// Shard configuration
#[derive(Debug, Clone)]
pub struct ShardConfig {
    pub shard_id: String,
    pub max_sessions: u32,
    pub max_capabilities: u32,
}