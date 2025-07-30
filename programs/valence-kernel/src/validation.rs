// Security validation layer for valence-kernel operation processing
//
// The valence-kernel acts as an "on-chain linker" that processes untrusted client data
// including account indices, CPI instruction data, and operation parameters. This validation
// module serves as the secure gatekeeper that prevents various classes of attack before
// operations reach the execution layer. This includes:
// - Resource exhaustion attacks through oversized data or unbounded collections
// - Buffer overflow attacks via malformed CPI data or operation parameters  
// - Out-of-bounds memory access through invalid account indices
// - Transaction griefing through excessively large payloads
//
// All client-provided data flows through these validators before the
// kernel's batch execution engine processes operations. This creates a secure boundary
// between untrusted external input and the kernel's trusted execution environment.
//
// Validation uses simple bounds checks and size limits to maintain Solana's
// strict performance requirements while providing comprehensive security coverage.
use crate::errors::KernelError;
use anchor_lang::prelude::*;

/// Maximum size for general operation data
pub const MAX_OPERATION_DATA_SIZE: usize = 1024;

/// Maximum size for custom operation data (may be different from general operations)
pub const MAX_CUSTOM_DATA_SIZE: usize = 512;

/// Maximum size for guard data passed to external programs
pub const MAX_GUARD_DATA_SIZE: usize = 256;

/// Maximum size for CPI instruction data
pub const MAX_CPI_DATA_SIZE: usize = 1024;

/// Generic data size validation
pub fn validate_data_size(data: &[u8], max_size: usize, error: KernelError) -> Result<()> {
    if data.len() > max_size {
        return Err(error.into());
    }
    Ok(())
}

/// Validate operation data size
pub fn validate_operation_data(data: &[u8]) -> Result<()> {
    validate_data_size(data, MAX_OPERATION_DATA_SIZE, KernelError::TransactionTooLarge)
}

/// Validate custom operation data size
pub fn validate_custom_data(data: &[u8]) -> Result<()> {
    validate_data_size(data, MAX_CUSTOM_DATA_SIZE, KernelError::TransactionTooLarge)
}

/// Validate guard data size
pub fn validate_guard_data(data: &[u8]) -> Result<()> {
    validate_data_size(data, MAX_GUARD_DATA_SIZE, KernelError::GuardDataTooLarge)
}

/// Validate CPI instruction data size
pub fn validate_cpi_data(data: &[u8]) -> Result<()> {
    validate_data_size(data, MAX_CPI_DATA_SIZE, KernelError::TransactionTooLarge)
}

/// Validate account indices are within bounds
pub fn validate_account_indices(indices: &[u8], max_index: u8) -> Result<()> {
    for &index in indices {
        if index > max_index {
            return Err(KernelError::InvalidParameters.into());
        }
    }
    Ok(())
}