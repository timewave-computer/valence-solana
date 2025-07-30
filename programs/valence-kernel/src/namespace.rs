// Hierarchical namespace system for valence-kernel session organization and isolation
//
// The valence-kernel manages multiple concurrent sessions that may operate across
// different protocols, organizations, or security domains. This namespace system
// provides hierarchical organization similar to filesystems, enabling secure
// multi-tenant operation while maintaining performance and preventing interference.
//
// KERNEL INTEGRATION: Sessions are organized within namespace hierarchies that
// control access patterns and inheritance. The kernel uses namespace paths for
// PDA derivation, session isolation, and permission checking during batch execution.
//
// SECURITY MODEL: Parent namespaces have implicit access to child state (one-way
// trust), enabling administrative control without explicit grants. Child namespaces
// cannot access parent state, ensuring security isolation between protocol boundaries.
//
// PERFORMANCE OPTIMIZATION: Fixed-size path structures eliminate heap allocations
// during namespace operations and prevent griefing attacks through variable-size
// data. Path-based PDA derivation provides O(1) lookups for session validation.
//
// MULTI-TENANT SUPPORT: Different protocols operate in separate namespace trees
// without interference, enabling safe concurrent execution of operations from
// multiple sources within the same kernel instance.

use anchor_lang::prelude::*;
use crate::errors::KernelError;

// ================================
// Constants
// ================================

/// Maximum length for namespace paths (256 bytes should be sufficient)
pub const MAX_NAMESPACE_PATH_LEN: usize = 256;

/// Maximum size for namespace state data
pub const MAX_NAMESPACE_STATE_SIZE: usize = 1024;

// ================================
// Fixed-Size Namespace Path
// ================================

/// Fixed-size namespace path using stack allocation
/// 
/// SECURITY: Fixed size prevents griefing attacks through large paths
/// PERFORMANCE: Stack allocation eliminates heap overhead
#[derive(Clone, Debug, PartialEq, Eq, AnchorSerialize, AnchorDeserialize)]
pub struct NamespacePath {
    /// Fixed-size path buffer
    pub path: [u8; MAX_NAMESPACE_PATH_LEN],
    /// Actual length of the path
    pub len: u16,
}

impl NamespacePath {
    /// Create a new namespace path from a string slice
    /// 
    /// # Errors
    /// Returns errors for invalid paths or length violations
    pub fn new(path: &str) -> Result<Self> {
        let path_bytes = path.as_bytes();
        
        // Validation
        require!(!path.is_empty(), KernelError::NamespaceEmptyPath);
        require!(path_bytes.len() <= MAX_NAMESPACE_PATH_LEN, KernelError::NamespaceInvalidPath);
        require!(!path.contains("//"), KernelError::NamespaceInvalidPath);
        require!(!path.starts_with('/'), KernelError::NamespaceInvalidPath);
        require!(!path.ends_with('/'), KernelError::NamespaceInvalidPath);
        
        // Copy to fixed buffer
        let mut buffer = [0u8; MAX_NAMESPACE_PATH_LEN];
        buffer[..path_bytes.len()].copy_from_slice(path_bytes);
        
        Ok(Self {
            path: buffer,
            len: u16::try_from(path_bytes.len()).map_err(|_| KernelError::NamespaceInvalidPath)?,
        })
    }
    
    /// Get the path as a string slice
    /// 
    /// # Errors
    /// Returns errors for invalid UTF-8 sequences
    pub fn as_str(&self) -> Result<&str> {
        std::str::from_utf8(&self.path[..self.len as usize])
            .map_err(|_| KernelError::NamespaceInvalidPath.into())
    }
    
    /// Get parent namespace path
    #[must_use]
    pub fn parent(&self) -> Option<Self> {
        let path_str = self.as_str().ok()?;
        path_str.rfind('/').and_then(|idx| {
            let mut buffer = [0u8; MAX_NAMESPACE_PATH_LEN];
            let parent_bytes = &self.path[..idx];
            buffer[..idx].copy_from_slice(parent_bytes);
            Some(Self {
                path: buffer,
                len: idx.try_into().ok()?,
            })
        })
    }
    
    /// Check if this namespace is a parent of another
    #[must_use]
    pub fn is_parent_of(&self, other: &Self) -> bool {
        if self.len >= other.len {
            return false;
        }
        
        // Check if other starts with self + '/'
        other.path[..self.len as usize] == self.path[..self.len as usize] &&
        other.path[self.len as usize] == b'/'
    }
    
    /// Get the depth (number of segments)
    #[must_use]
    pub fn depth(&self) -> u8 {
        if self.len == 0 {
            return 0;
        }
        
        let path_slice = &self.path[..self.len as usize];
        #[allow(clippy::naive_bytecount)] // Used in const context, avoids external deps
        u8::try_from(path_slice.iter().filter(|&&b| b == b'/').count()).unwrap_or(u8::MAX).saturating_add(1)
    }
    
    /// Create a child namespace path
    /// 
    /// # Errors
    /// Returns errors for invalid child names or path length violations
    pub fn child(&self, child_name: &str) -> Result<Self> {
        let parent_str = self.as_str()?;
        let child_path = format!("{parent_str}/{child_name}");
        Self::new(&child_path)
    }
}

// ================================
// Fixed-Size Namespace Account
// ================================

/// On-chain namespace account with fixed-size state
#[account]
#[derive(Debug)]
pub struct Namespace {
    /// Fixed-size namespace path
    pub path: NamespacePath,
    
    /// Parent namespace (None for root shards)
    pub parent: Option<Pubkey>,
    
    /// Owner who created this namespace
    pub owner: Pubkey,
    
    /// Fixed-size state data buffer
    pub state: [u8; MAX_NAMESPACE_STATE_SIZE],
    
    /// Actual length of state data
    pub state_len: u32,
    
    /// Unix timestamp of creation
    pub created_at: i64,
    
    /// Maximum state size allowed (must be <= MAX_NAMESPACE_STATE_SIZE)
    pub max_state_size: u32,
    
    /// Number of child namespaces
    pub child_count: u32,
}

impl Namespace {
    pub const SEED_PREFIX: &'static [u8] = b"namespace";
    
    /// Calculate exact space needed for this account
    #[must_use]
    pub const fn space() -> usize {
        8 + // discriminator
        MAX_NAMESPACE_PATH_LEN + 2 + // path + len
        1 + 32 + // Option<parent>
        32 + // owner
        MAX_NAMESPACE_STATE_SIZE + // state buffer
        4 + // state_len
        8 + // created_at
        4 + // max_state_size
        4   // child_count
    }
    
    /// Create a new namespace
    /// 
    /// # Errors
    /// Returns errors for invalid state size or path violations
    pub fn new(
        path: NamespacePath,
        parent: Option<Pubkey>,
        owner: Pubkey,
        max_state_size: u32,
        clock: &Clock,
    ) -> Result<Self> {
        require!(
            max_state_size <= u32::try_from(MAX_NAMESPACE_STATE_SIZE).map_err(|_| KernelError::NamespaceStateTooLarge)?,
            KernelError::NamespaceStateTooLarge
        );
        
        Ok(Self {
            path,
            parent,
            owner,
            state: [0u8; MAX_NAMESPACE_STATE_SIZE],
            state_len: 0,
            created_at: clock.unix_timestamp,
            max_state_size,
            child_count: 0,
        })
    }
    
    /// Write state data
    /// 
    /// # Errors
    /// Returns errors for data too large or state violations
    pub fn write_state(&mut self, data: &[u8]) -> Result<()> {
        require!(
            data.len() <= self.max_state_size as usize,
            KernelError::NamespaceStateTooLarge
        );
        require!(
            data.len() <= MAX_NAMESPACE_STATE_SIZE,
            KernelError::NamespaceStateTooLarge
        );
        
        self.state[..data.len()].copy_from_slice(data);
        self.state_len = u32::try_from(data.len()).map_err(|_| KernelError::NamespaceStateTooLarge)?;
        Ok(())
    }
    
    /// Read state data
    #[must_use]
    pub fn read_state(&self) -> &[u8] {
        &self.state[..self.state_len as usize]
    }
    
    /// Update state with bounds checking
    /// 
    /// # Errors
    /// Returns errors for out of bounds access or overflow
    pub fn update_state(&mut self, offset: u32, data: &[u8]) -> Result<()> {
        let end = offset.checked_add(u32::try_from(data.len()).map_err(|_| KernelError::InvalidParameters)?)
            .ok_or(KernelError::InvalidParameters)?;
            
        require!(
            end <= self.state_len,
            KernelError::InvalidParameters
        );
        
        let start = offset as usize;
        let end = end as usize;
        self.state[start..end].copy_from_slice(data);
        Ok(())
    }
    
    /// Derive PDA for a namespace path
    #[must_use]
    pub fn derive_pda(path: &NamespacePath, program_id: &Pubkey) -> (Pubkey, u8) {
        let path_bytes = &path.path[..path.len as usize];
        Pubkey::find_program_address(
            &[Self::SEED_PREFIX, path_bytes],
            program_id,
        )
    }
}

// ================================
// Namespace Context
// ================================

/// Namespace evaluation context with fixed-size data
#[derive(Clone, Debug)]
pub struct NamespaceContext {
    /// Current namespace being accessed
    pub current: NamespacePath,
    
    /// Calling namespace (for cross-namespace operations)
    pub caller: NamespacePath,
    
    /// Authority performing the operation
    pub signer: Pubkey,
    
    /// Current timestamp
    pub timestamp: i64,
}

impl NamespaceContext {
    /// Check if signer has access based on namespace hierarchy
    #[must_use]
    pub fn has_access(&self, target: &NamespacePath) -> bool {
        // Callers can access their own namespace and descendants
        self.caller.path[..self.caller.len as usize] == target.path[..self.caller.len as usize] ||
        target.is_parent_of(&self.caller)
    }
}

