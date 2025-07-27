use crate::error::{RegistryError, Result};
use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, RwLock};
// Note: Using simplified SMT implementation due to API compatibility issues
// In production, would use proper sparse-merkle-tree implementation

// ================================
// Storage Backend Trait
// ================================

#[async_trait]
pub trait StorageBackend: Send + Sync {
    /// Store content and return hash
    async fn store(&self, content: &[u8]) -> Result<String>;

    /// Retrieve content by hash
    async fn retrieve(&self, hash: &str) -> Result<Vec<u8>>;

    /// Check if content exists
    async fn exists(&self, hash: &str) -> Result<bool>;

    /// Delete content
    async fn delete(&self, hash: &str) -> Result<()>;
}

// ================================
// Local File Storage
// ================================

pub struct LocalStorage {
    base_path: std::path::PathBuf,
}

impl LocalStorage {
    pub fn new<P: AsRef<Path>>(base_path: P) -> Result<Self> {
        let base_path = base_path.as_ref().to_path_buf();
        std::fs::create_dir_all(&base_path)
            .map_err(|e| RegistryError::StorageError(e.to_string()))?;

        Ok(Self { base_path })
    }

    fn get_path(&self, hash: &str) -> std::path::PathBuf {
        self.base_path.join(hash)
    }
}

#[async_trait]
impl StorageBackend for LocalStorage {
    async fn store(&self, content: &[u8]) -> Result<String> {
        let hash = blake3::hash(content);
        let hash_str = hex::encode(hash.as_bytes());
        let path = self.get_path(&hash_str);

        tokio::fs::write(&path, content)
            .await
            .map_err(|e| RegistryError::StorageError(e.to_string()))?;

        Ok(hash_str)
    }

    async fn retrieve(&self, hash: &str) -> Result<Vec<u8>> {
        let path = self.get_path(hash);
        tokio::fs::read(&path)
            .await
            .map_err(|e| RegistryError::StorageError(e.to_string()))
    }

    async fn exists(&self, hash: &str) -> Result<bool> {
        let path = self.get_path(hash);
        Ok(path.exists())
    }

    async fn delete(&self, hash: &str) -> Result<()> {
        let path = self.get_path(hash);
        tokio::fs::remove_file(&path)
            .await
            .map_err(|e| RegistryError::StorageError(e.to_string()))
    }
}

// ================================
// In-Memory Storage Backend
// ================================

/// In-memory storage backend for development and testing
pub struct InMemoryStorage {
    data: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl StorageBackend for InMemoryStorage {
    async fn store(&self, content: &[u8]) -> Result<String> {
        let hash = blake3::hash(content);
        let hash_str = hex::encode(hash.as_bytes());

        let mut data = self.data.write().unwrap();
        data.insert(hash_str.clone(), content.to_vec());

        Ok(hash_str)
    }

    async fn retrieve(&self, hash: &str) -> Result<Vec<u8>> {
        let data = self.data.read().unwrap();
        data.get(hash)
            .cloned()
            .ok_or_else(|| RegistryError::StorageError(format!("Content not found: {}", hash)))
    }

    async fn exists(&self, hash: &str) -> Result<bool> {
        let data = self.data.read().unwrap();
        Ok(data.contains_key(hash))
    }

    async fn delete(&self, hash: &str) -> Result<()> {
        let mut data = self.data.write().unwrap();
        data.remove(hash);
        Ok(())
    }
}

// ================================
// SMT Tree for Registry Objects
// ================================

/// Simplified Sparse Merkle Tree for tracking registry objects
/// Note: This is a simplified implementation. In production, would use a proper SMT crate
pub struct RegistrySMT {
    /// Storage for key-value pairs
    nodes: HashMap<[u8; 32], [u8; 32]>, // key -> value_hash
    /// Current root hash
    root: [u8; 32],
}

impl Default for RegistrySMT {
    fn default() -> Self {
        Self::new()
    }
}

impl RegistrySMT {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            root: [0u8; 32],
        }
    }

    /// Insert a key-value pair into the SMT
    pub fn insert(&mut self, key: [u8; 32], value: Vec<u8>) -> Result<[u8; 32]> {
        let value_hash: [u8; 32] = blake3::hash(&value).into();
        self.nodes.insert(key, value_hash);
        self.root = self.compute_root();
        Ok(self.root)
    }

    /// Get a value hash from the SMT
    pub fn get(&self, key: [u8; 32]) -> Option<[u8; 32]> {
        self.nodes.get(&key).copied()
    }

    /// Generate a merkle proof for inclusion (simplified)
    pub fn generate_proof(&self, keys: Vec<[u8; 32]>) -> Result<Vec<u8>> {
        // Simplified proof - just include the root and existence checks
        let mut proof = Vec::new();
        proof.extend_from_slice(&self.root);

        // Add existence flags for each key
        for key in keys {
            proof.push(if self.nodes.contains_key(&key) { 1 } else { 0 });
        }

        Ok(proof)
    }

    /// Verify a merkle proof (simplified)
    pub fn verify_proof(
        &self,
        keys: Vec<[u8; 32]>,
        values: Vec<[u8; 32]>,
        proof_data: &[u8],
    ) -> Result<bool> {
        if keys.len() != values.len() || proof_data.len() < 32 {
            return Ok(false);
        }

        // Check if root matches
        let proof_root: [u8; 32] = proof_data[0..32].try_into().unwrap_or([0u8; 32]);
        if proof_root != self.root {
            return Ok(false);
        }

        // Verify each key-value pair exists
        for (i, (key, expected_value)) in keys.into_iter().zip(values).enumerate() {
            if let Some(stored_value) = self.get(key) {
                if stored_value != expected_value {
                    return Ok(false);
                }
            } else if proof_data.get(32 + i) != Some(&0) {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Get current root hash
    pub fn root(&self) -> [u8; 32] {
        self.root
    }

    /// Get root as byte array (same as root for compatibility)
    pub fn root_bytes(&self) -> [u8; 32] {
        self.root
    }

    /// Compute the current root hash
    fn compute_root(&self) -> [u8; 32] {
        let mut hasher = blake3::Hasher::new();

        // Create a sorted vector of (key, value_hash) pairs for deterministic hashing
        let mut sorted_entries: Vec<_> = self.nodes.iter().collect();
        sorted_entries.sort_by_key(|(k, _)| *k);

        for (key, value_hash) in sorted_entries {
            hasher.update(key);
            hasher.update(value_hash);
        }

        hasher.finalize().into()
    }
}

// ================================
// Caching Layer
// ================================

pub struct CachedStorage<B: StorageBackend> {
    backend: B,
    cache: std::sync::RwLock<lru::LruCache<String, Vec<u8>>>,
}

impl<B: StorageBackend> CachedStorage<B> {
    pub fn new(backend: B, cache_size: usize) -> Self {
        Self {
            backend,
            cache: std::sync::RwLock::new(lru::LruCache::new(
                std::num::NonZeroUsize::new(cache_size).unwrap(),
            )),
        }
    }
}

#[async_trait]
impl<B: StorageBackend> StorageBackend for CachedStorage<B> {
    async fn store(&self, content: &[u8]) -> Result<String> {
        let hash = self.backend.store(content).await?;

        // Add to cache
        let mut cache = self.cache.write().unwrap();
        cache.put(hash.clone(), content.to_vec());

        Ok(hash)
    }

    async fn retrieve(&self, hash: &str) -> Result<Vec<u8>> {
        // Check cache first
        {
            let mut cache = self.cache.write().unwrap();
            if let Some(content) = cache.get(hash) {
                return Ok(content.clone());
            }
        }

        // Fetch from backend
        let content = self.backend.retrieve(hash).await?;

        // Add to cache
        {
            let mut cache = self.cache.write().unwrap();
            cache.put(hash.to_string(), content.clone());
        }

        Ok(content)
    }

    async fn exists(&self, hash: &str) -> Result<bool> {
        // Check cache first
        {
            let cache = self.cache.read().unwrap();
            if cache.contains(hash) {
                return Ok(true);
            }
        }

        self.backend.exists(hash).await
    }

    async fn delete(&self, hash: &str) -> Result<()> {
        // Remove from cache
        {
            let mut cache = self.cache.write().unwrap();
            cache.pop(hash);
        }

        self.backend.delete(hash).await
    }
}
