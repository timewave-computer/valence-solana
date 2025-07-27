use crate::error::{RegistryError, Result};
use blake3;
use serde::{Deserialize, Serialize};
use valence_functions::function_core::FunctionMetadata;

// ================================
// Function Registry
// ================================

/// Function registry entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionEntry {
    /// Content-addressed hash
    pub content_hash: [u8; 32],

    /// Function metadata
    pub metadata: FunctionMetadata,

    /// Source code (Rust)
    pub source_code: String,

    /// Compiled WASM (optional)
    pub wasm_bytecode: Option<Vec<u8>>,

    /// Registration timestamp
    pub registered_at: i64,

    /// Registry version
    pub registry_version: u16,

    /// Tags for discovery
    pub tags: Vec<String>,
}

impl FunctionEntry {
    /// Create new function entry
    pub fn new(metadata: FunctionMetadata, source_code: String, tags: Vec<String>) -> Self {
        let content_hash = Self::compute_hash(&source_code);

        Self {
            content_hash,
            metadata,
            source_code,
            wasm_bytecode: None,
            registered_at: chrono::Utc::now().timestamp(),
            registry_version: 1,
            tags,
        }
    }

    /// Compute content hash
    pub fn compute_hash(source_code: &str) -> [u8; 32] {
        blake3::hash(source_code.as_bytes()).into()
    }

    /// Verify content hash
    pub fn verify_hash(&self) -> bool {
        self.content_hash == Self::compute_hash(&self.source_code)
    }
}

// ================================
// Function Registry Service
// ================================

#[async_trait::async_trait]
pub trait FunctionRegistry: Send + Sync {
    /// Register a new function
    async fn register_function(&self, entry: FunctionEntry) -> Result<[u8; 32]>;

    /// Get function by content hash
    async fn get_function(&self, hash: &[u8; 32]) -> Result<FunctionEntry>;

    /// Search functions by tags
    async fn search_by_tags(&self, tags: &[String]) -> Result<Vec<FunctionEntry>>;

    /// List all functions
    async fn list_functions(&self, limit: usize, offset: usize) -> Result<Vec<FunctionEntry>>;

    /// Get function dependencies
    async fn get_dependencies(&self, hash: &[u8; 32]) -> Result<Vec<[u8; 32]>>;
}

// ================================
// In-Memory Function Registry
// ================================

pub struct InMemoryFunctionRegistry {
    functions: std::sync::RwLock<std::collections::HashMap<[u8; 32], FunctionEntry>>,
}

impl Default for InMemoryFunctionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryFunctionRegistry {
    pub fn new() -> Self {
        Self {
            functions: std::sync::RwLock::new(std::collections::HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl FunctionRegistry for InMemoryFunctionRegistry {
    async fn register_function(&self, entry: FunctionEntry) -> Result<[u8; 32]> {
        if !entry.verify_hash() {
            return Err(RegistryError::InvalidContentHash);
        }

        let hash = entry.content_hash;
        let mut functions = self.functions.write().unwrap();

        if functions.contains_key(&hash) {
            return Err(RegistryError::FunctionAlreadyExists(hex::encode(hash)));
        }

        functions.insert(hash, entry);
        Ok(hash)
    }

    async fn get_function(&self, hash: &[u8; 32]) -> Result<FunctionEntry> {
        let functions = self.functions.read().unwrap();
        functions
            .get(hash)
            .cloned()
            .ok_or_else(|| RegistryError::FunctionNotFound(hex::encode(hash)))
    }

    async fn search_by_tags(&self, tags: &[String]) -> Result<Vec<FunctionEntry>> {
        let functions = self.functions.read().unwrap();
        let results: Vec<_> = functions
            .values()
            .filter(|entry| tags.iter().any(|tag| entry.tags.contains(tag)))
            .cloned()
            .collect();
        Ok(results)
    }

    async fn list_functions(&self, limit: usize, offset: usize) -> Result<Vec<FunctionEntry>> {
        let functions = self.functions.read().unwrap();
        let results: Vec<_> = functions
            .values()
            .skip(offset)
            .take(limit)
            .cloned()
            .collect();
        Ok(results)
    }

    async fn get_dependencies(&self, hash: &[u8; 32]) -> Result<Vec<[u8; 32]>> {
        let entry = self.get_function(hash).await?;
        Ok(entry.metadata.dependencies)
    }
}
