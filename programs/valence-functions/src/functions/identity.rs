// Identity function - returns input unchanged
// Registry ID: 1001
// Purpose: Testing and development

use anchor_lang::prelude::*;

/// Simple identity function that returns the input value unchanged
/// 
/// This function is primarily used for testing the function registry system
/// and as a template for other functions.
#[allow(clippy::needless_pass_by_value)]
pub fn identity(input: u64) -> Result<u64> {
    msg!("Identity function called with input: {}", input);
    Ok(input)
}

/// Metadata for function registry
pub const FUNCTION_ID: u64 = 1001;
pub const FUNCTION_NAME: &str = "identity";
pub const FUNCTION_VERSION: u16 = 1;
pub const COMPUTE_UNITS: u64 = 1_000;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity() {
        assert_eq!(identity(42).unwrap(), 42);
        assert_eq!(identity(0).unwrap(), 0);
        assert_eq!(identity(u64::MAX).unwrap(), u64::MAX);
    }
}