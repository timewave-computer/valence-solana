// Generic bitmap implementation for efficient bit tracking
use anchor_lang::prelude::*;

/// Generic bitmap for tracking occupied/free slots
#[derive(Debug, Clone, Copy, AnchorSerialize, AnchorDeserialize)]
pub struct BitMap<const N: usize> {
    /// Bit storage (supports up to N*8 slots)
    storage: [u8; N],
}

impl<const N: usize> Default for BitMap<N> {
    fn default() -> Self {
        Self { storage: [0; N] }
    }
}

impl<const N: usize> BitMap<N> {
    /// Maximum number of slots this bitmap can track
    pub const CAPACITY: usize = N * 8;
    
    /// Create a new empty bitmap
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Check if a slot is set
    pub fn is_set(&self, index: usize) -> bool {
        if index >= Self::CAPACITY {
            return false;
        }
        let byte_idx = index / 8;
        let bit_idx = index % 8;
        (self.storage[byte_idx] & (1 << bit_idx)) != 0
    }
    
    /// Set a slot
    pub fn set(&mut self, index: usize) -> Result<()> {
        if index >= Self::CAPACITY {
            return Err(crate::errors::ValenceError::InvalidParameters.into());
        }
        let byte_idx = index / 8;
        let bit_idx = index % 8;
        self.storage[byte_idx] |= 1 << bit_idx;
        Ok(())
    }
    
    /// Clear a slot
    pub fn clear(&mut self, index: usize) -> Result<()> {
        if index >= Self::CAPACITY {
            return Err(crate::errors::ValenceError::InvalidParameters.into());
        }
        let byte_idx = index / 8;
        let bit_idx = index % 8;
        self.storage[byte_idx] &= !(1 << bit_idx);
        Ok(())
    }
    
    /// Find first free slot
    pub fn first_free(&self) -> Option<usize> {
        for (byte_idx, &byte) in self.storage.iter().enumerate() {
            if byte != 0xFF {
                let bit_idx = (!byte).trailing_zeros() as usize;
                return Some(byte_idx * 8 + bit_idx);
            }
        }
        None
    }
    
    /// Count occupied slots
    pub fn count_set(&self) -> usize {
        self.storage.iter().map(|&b| b.count_ones() as usize).sum()
    }
    
    /// Check if all slots are occupied
    pub fn is_full(&self) -> bool {
        self.storage.iter().all(|&b| b == 0xFF)
    }
    
    /// Check if all slots are free
    pub fn is_empty(&self) -> bool {
        self.storage.iter().all(|&b| b == 0)
    }
}

/// Type alias for single-byte bitmap (8 slots)
pub type BitMap8 = BitMap<1>;