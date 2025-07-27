// Centralized validation utilities for data size and security checks
use crate::errors::ValenceError;
use anchor_lang::prelude::*;

/// Maximum size for general operation data
pub const MAX_OPERATION_DATA_SIZE: usize = 1024;

/// Maximum size for custom operation data (may be different from general operations)
pub const MAX_CUSTOM_DATA_SIZE: usize = 512;

/// Maximum size for guard data passed to external programs
pub const MAX_GUARD_DATA_SIZE: usize = 256;

/// Maximum size for CPI instruction data
pub const MAX_CPI_DATA_SIZE: usize = 1024;

/// Validate that data does not exceed the maximum operation size
pub fn validate_operation_data(data: &[u8]) -> Result<()> {
    require!(
        data.len() <= MAX_OPERATION_DATA_SIZE,
        ValenceError::TransactionTooLarge
    );
    Ok(())
}

/// Validate custom operation data size
pub fn validate_custom_data(data: &[u8]) -> Result<()> {
    require!(
        data.len() <= MAX_CUSTOM_DATA_SIZE,
        ValenceError::TransactionTooLarge
    );
    Ok(())
}

/// Validate guard data size
pub fn validate_guard_data(data: &[u8]) -> Result<()> {
    require!(
        data.len() <= MAX_GUARD_DATA_SIZE,
        ValenceError::GuardDataTooLarge
    );
    Ok(())
}

/// Validate CPI instruction data size
pub fn validate_cpi_data(data: &[u8]) -> Result<()> {
    require!(
        data.len() <= MAX_CPI_DATA_SIZE,
        ValenceError::TransactionTooLarge
    );
    Ok(())
}

/// Validate a vector of data items doesn't exceed maximum count
pub fn validate_vec_count<T>(vec: &[T], max_count: usize) -> Result<()> {
    require!(
        vec.len() <= max_count,
        ValenceError::TransactionTooLarge
    );
    Ok(())
}

/// Validate account indices are within bounds
pub fn validate_account_indices(indices: &[u8], max_index: u8) -> Result<()> {
    for &index in indices {
        require!(
            index <= max_index,
            ValenceError::InvalidParameters
        );
    }
    Ok(())
}