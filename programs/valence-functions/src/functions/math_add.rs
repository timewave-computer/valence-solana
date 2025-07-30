// Mathematical addition function with overflow protection
// Registry ID: 1002
// Purpose: Safe integer addition

use anchor_lang::prelude::*;

/// Error type for math operations
#[error_code]
pub enum MathError {
    #[msg("Integer overflow")]
    Overflow,
}

/// Input for addition operation
#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct AddInput {
    pub a: u64,
    pub b: u64,
}

/// Safe addition function that prevents overflow
/// 
/// This function performs checked addition to prevent integer overflow attacks.
/// Returns an error if the operation would overflow.
#[allow(clippy::needless_pass_by_value)]
pub fn math_add(input: AddInput) -> Result<u64> {
    msg!("Adding {} + {}", input.a, input.b);
    
    input.a
        .checked_add(input.b)
        .ok_or(MathError::Overflow.into())
}

/// Metadata for function registry
pub const FUNCTION_ID: u64 = 1002;
pub const FUNCTION_NAME: &str = "math_add";
pub const FUNCTION_VERSION: u16 = 1;
pub const COMPUTE_UNITS: u64 = 2_000;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_math_add_normal() {
        let input = AddInput { a: 10, b: 20 };
        assert_eq!(math_add(input).unwrap(), 30);
    }

    #[test]
    fn test_math_add_overflow() {
        let input = AddInput { a: u64::MAX, b: 1 };
        assert!(math_add(input).is_err());
    }

    #[test]
    fn test_math_add_zero() {
        let input = AddInput { a: 42, b: 0 };
        assert_eq!(math_add(input).unwrap(), 42);
    }
}