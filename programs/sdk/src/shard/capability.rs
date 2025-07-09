/// Capability management for shards


/// Capability configuration
#[derive(Debug, Clone)]
pub struct CapabilityConfig {
    pub capability_id: String,
    pub verification_functions: Vec<String>,
    pub description: String,
}