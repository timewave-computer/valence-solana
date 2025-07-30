// Bump allocator for valence-kernel operation processing
//
// The valence-kernel processes batches of operations (account borrowing, CPI calls, etc.) 
// that require temporary data structures during execution. The bump allocator provides
// zero-heap-allocation temporary storage for building operation lists, account indices,
// and validation buffers.
//
// During batch execution, the kernel acts as an "on-chain linker" that 
// takes flat account lists and operation indices, then builds temporary data structures 
// for validation and execution. This allocator ensures all temporary allocations are 
// stack-based and automatically freed when the instruction completes.
// 
// USAGE: Create a BumpAllocator at the start of batch processing instruction handlers,
// use it for all temporary allocations during operation processing, and it automatically 
// cleans up when dropped at the end of the instruction.

use anchor_lang::prelude::*;
use core::mem::{size_of, align_of};

/// Fixed-size bump allocator for transaction-scoped data
/// 
/// DESIGN: Uses a fixed-size buffer on the stack and bumps a pointer
/// for each allocation. All allocations are freed when the allocator
/// is dropped (at instruction end).
pub struct BumpAllocator<const SIZE: usize> {
    buffer: [u8; SIZE],
    offset: usize,
}

impl<const SIZE: usize> Default for BumpAllocator<SIZE> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const SIZE: usize> BumpAllocator<SIZE> {
    /// Create a new bump allocator
    pub const fn new() -> Self {
        Self {
            buffer: [0u8; SIZE],
            offset: 0,
        }
    }
    
    /// Allocate space for T, returning a mutable reference
    /// 
    /// SAFETY: The returned reference is valid only for the lifetime
    /// of the allocator. Do not store these references in accounts!
    pub fn alloc<T>(&mut self) -> Option<&mut T> {
        let align = align_of::<T>();
        let size = size_of::<T>();
        
        // Align the offset
        let aligned_offset = (self.offset + align - 1) & !(align - 1);
        
        // Check if we have space
        if aligned_offset + size > SIZE {
            return None;
        }
        
        // Get pointer to aligned location
        let ptr = unsafe {
            self.buffer.as_mut_ptr().add(aligned_offset) as *mut T
        };
        
        // Update offset
        self.offset = aligned_offset + size;
        
        // Return reference
        unsafe { Some(&mut *ptr) }
    }
    
    /// Allocate space for a slice
    pub fn alloc_slice<T>(&mut self, len: usize) -> Option<&mut [T]> {
        let align = align_of::<T>();
        let size = size_of::<T>() * len;
        
        // Align the offset
        let aligned_offset = (self.offset + align - 1) & !(align - 1);
        
        // Check if we have space
        if aligned_offset + size > SIZE {
            return None;
        }
        
        // Get pointer to aligned location
        let ptr = unsafe {
            self.buffer.as_mut_ptr().add(aligned_offset) as *mut T
        };
        
        // Update offset
        self.offset = aligned_offset + size;
        
        // Return slice
        unsafe { Some(core::slice::from_raw_parts_mut(ptr, len)) }
    }
    
    /// Reset the allocator (frees all allocations)
    pub fn reset(&mut self) {
        self.offset = 0;
    }
    
    /// Get remaining capacity
    pub const fn remaining(&self) -> usize {
        SIZE - self.offset
    }
}

/// Transaction-scoped allocator with 4KB of space
/// 
/// This is suitable for most transaction needs without being
/// too large for the stack.
pub type TransactionAllocator = BumpAllocator<4096>;

/// Example: Temporary operation buffer using bump allocation
pub struct TempOperationBuffer<'a> {
    operations: &'a mut [KernelOperation],
    count: usize,
}

impl<'a> TempOperationBuffer<'a> {
    /// Create a new buffer using the allocator
    pub fn new(allocator: &'a mut TransactionAllocator, capacity: usize) -> Option<Self> {
        let operations = allocator.alloc_slice::<KernelOperation>(capacity)?;
        Some(Self {
            operations,
            count: 0,
        })
    }
    
    /// Add an operation
    pub fn push(&mut self, op: KernelOperation) -> Result<()> {
        if self.count >= self.operations.len() {
            return Err(error!(KernelError::TransactionTooLarge));
        }
        self.operations[self.count] = op;
        self.count += 1;
        Ok(())
    }
    
    /// Get operations as slice
    pub fn as_slice(&self) -> &[KernelOperation] {
        &self.operations[..self.count]
    }
}

// Re-export necessary types
use crate::instructions::batch_operations::KernelOperation;
use crate::errors::KernelError;

/// Example usage in an instruction handler:
/// ```no_run
/// pub fn example_handler(ctx: Context<Example>) -> Result<()> {
///     // Create allocator at start of instruction
///     let mut allocator = TransactionAllocator::new();
///     
///     // Use it for temporary allocations
///     let mut ops = TempOperationBuffer::new(&mut allocator, 16)
///         .ok_or(KernelError::TransactionTooLarge)?;
///     
///     ops.push(KernelOperation::BorrowAccount {
///         account: ctx.accounts.some_account.key(),
///         mode: ACCESS_MODE_READ,
///     })?;
///     
///     // ... use ops ...
///     
///     // Allocator and all allocations are automatically freed here
///     Ok(())
/// }
/// ```

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_bump_allocator() {
        let mut allocator = BumpAllocator::<1024>::new();
        
        // Allocate a u64
        let num = allocator.alloc::<u64>().unwrap();
        *num = 42;
        assert_eq!(*num, 42);
        
        // Allocate an array
        let arr = allocator.alloc_slice::<u32>(10).unwrap();
        arr[0] = 100;
        arr[9] = 900;
        assert_eq!(arr[0], 100);
        assert_eq!(arr[9], 900);
        
        // Check remaining capacity
        let used = size_of::<u64>() + (size_of::<u32>() * 10);
        assert!(allocator.remaining() <= 1024 - used);
    }
}