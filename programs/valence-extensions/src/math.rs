//! Extended math functionality for Valence
//! 64.64 bit fixed point: 64 bits integer, 64 bits fraction
//! Provides safe arithmetic operations and precision utilities

/// Fixed-point number with 64 bits of precision
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct FixedPoint(pub u128);

impl FixedPoint {
    /// One as fixed-point
    pub const ONE: Self = Self(1 << 64);
    
    /// Zero
    pub const ZERO: Self = Self(0);
    
    /// Create from integer
    pub fn from_int(n: u64) -> Self {
        Self((n as u128) << 64)
    }
    
    /// Convert to integer (truncates fraction)
    pub fn to_int(self) -> u64 {
        (self.0 >> 64) as u64
    }
    
    /// Multiply two fixed-point numbers
    #[allow(clippy::should_implement_trait)]
    pub fn mul(self, other: Self) -> Result<Self, MathError> {
        // For 64.64 fixed point: (a * 2^64) * (b * 2^64) / 2^64 = a * b * 2^64
        // We need to avoid overflow in the intermediate calculation
        
        // Extract integer and fractional parts
        let a_int = self.0 >> 64;
        let a_frac = self.0 & ((1u128 << 64) - 1);
        let b_int = other.0 >> 64;
        let b_frac = other.0 & ((1u128 << 64) - 1);
        
        // Calculate: a_int * b_int * 2^64 + a_int * b_frac + a_frac * b_int + (a_frac * b_frac) / 2^64
        let int_part = a_int.checked_mul(b_int).ok_or(MathError::Overflow)?;
        let cross1 = a_int.checked_mul(b_frac).ok_or(MathError::Overflow)?;
        let cross2 = a_frac.checked_mul(b_int).ok_or(MathError::Overflow)?;
        let frac_part = a_frac.checked_mul(b_frac).ok_or(MathError::Overflow)? >> 64;
        
        let result = (int_part << 64)
            .checked_add(cross1)
            .and_then(|x| x.checked_add(cross2))
            .and_then(|x| x.checked_add(frac_part))
            .ok_or(MathError::Overflow)?;
            
        Ok(Self(result))
    }
    
    /// Divide two fixed-point numbers
    #[allow(clippy::should_implement_trait)]
    pub fn div(self, other: Self) -> Result<Self, MathError> {
        if other.0 == 0 {
            return Err(MathError::DivisionByZero);
        }
        
        // For 64.64 fixed point division: (a * 2^64) / (b * 2^64) * 2^64 = (a / b) * 2^64
        // We need to calculate (self.0 * 2^64) / other.0 without overflow
        
        // Check if we can do the simple calculation without overflow
        if self.0 <= u128::MAX >> 64 {
            let dividend = self.0 << 64;
            Ok(Self(dividend / other.0))
        } else {
            // Use long division for larger numbers
            // This is a simplified version - for production use a more robust algorithm
            let quotient = self.0 / other.0;
            let remainder = self.0 % other.0;
            let fractional = (remainder << 64) / other.0;
            Ok(Self((quotient << 64) + fractional))
        }
    }
    
    /// Square root using Newton's method
    pub fn sqrt(self) -> Result<Self, MathError> {
        if self.0 == 0 {
            return Ok(Self::ZERO);
        }
        
        // For 64.64 fixed point sqrt, we need to be careful with the algorithm
        // Start with a reasonable initial guess (half the value)
        let mut x = self.0 >> 1;
        if x == 0 {
            x = 1 << 32; // Start with sqrt(1) as minimum guess
        }
        
        let mut prev = 0u128;
        
        // Newton's method: x_{n+1} = (x_n + self/x_n) / 2
        // We need to be careful about the division
        for _ in 0..50 { // Limit iterations to prevent infinite loops
            if x == prev {
                break;
            }
            prev = x;
            
            // Calculate self/x avoiding overflow
            let quotient = if self.0 <= u128::MAX >> 64 {
                (self.0 << 64) / x
            } else {
                // Fallback for very large numbers
                self.0 / (x >> 64)
            };
            
            x = (x + quotient) / 2;
        }
        
        Ok(Self(x))
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum MathError {
    Overflow,
    DivisionByZero,
}

impl From<MathError> for anchor_lang::error::Error {
    fn from(err: MathError) -> Self {
        match err {
            MathError::Overflow => anchor_lang::error::Error::from(anchor_lang::error::ErrorCode::AccountNotEnoughKeys),
            MathError::DivisionByZero => anchor_lang::error::Error::from(anchor_lang::error::ErrorCode::AccountNotEnoughKeys),
        }
    }
}

/// Safe arithmetic helpers
impl FixedPoint {
    /// Add multiple values with overflow checking
    pub fn sum_checked(values: &[FixedPoint]) -> Result<Self, MathError> {
        values.iter().try_fold(Self::ZERO, |acc, &val| {
            acc.0.checked_add(val.0)
                .map(Self)
                .ok_or(MathError::Overflow)
        })
    }
    
    /// Calculate the minimum of multiple values
    pub fn min_of(values: &[FixedPoint]) -> Option<Self> {
        values.iter().min().copied()
    }
    
    /// Calculate the maximum of multiple values
    pub fn max_of(values: &[FixedPoint]) -> Option<Self> {
        values.iter().max().copied()
    }
}

/// Precision utilities
pub mod precision {
    use super::*;
    
    /// Round to specific decimal places
    pub fn round_to_decimals(value: FixedPoint, decimals: u8) -> FixedPoint {
        if decimals >= 19 {
            // More than 19 decimals is beyond our precision
            return value;
        }
        
        // Calculate the mask for rounding
        let decimal_bits = 64 - (decimals as u32 * 3322 / 1000); // log2(10) ≈ 3.322
        if decimal_bits >= 64 {
            return FixedPoint::ZERO;
        }
        
        let mask = !((1u128 << decimal_bits) - 1);
        let rounded = value.0 & mask;
        
        // Check if we need to round up
        let remainder = value.0 & !mask;
        let half = 1u128 << (decimal_bits - 1);
        
        if remainder >= half {
            FixedPoint(rounded.saturating_add(1u128 << decimal_bits))
        } else {
            FixedPoint(rounded)
        }
    }
    
    /// Convert between different decimal precisions
    pub fn adjust_decimals(
        value: u64,
        from_decimals: u8,
        to_decimals: u8,
    ) -> Result<u64, MathError> {
        match from_decimals.cmp(&to_decimals) {
            std::cmp::Ordering::Equal => Ok(value),
            std::cmp::Ordering::Less => {
                // Scale up
                let scale = 10u64.checked_pow((to_decimals - from_decimals) as u32)
                    .ok_or(MathError::Overflow)?;
                value.checked_mul(scale).ok_or(MathError::Overflow)
            }
            std::cmp::Ordering::Greater => {
                // Scale down
                let scale = 10u64.pow((from_decimals - to_decimals) as u32);
                Ok(value / scale)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_ops() {
        let two = FixedPoint::from_int(2);
        let three = FixedPoint::from_int(3);
        
        assert_eq!(two.mul(three).unwrap().to_int(), 6);
        assert_eq!(three.div(two).unwrap().to_int(), 1); // 3/2 = 1.5, truncated
        assert_eq!(FixedPoint::from_int(4).sqrt().unwrap().to_int(), 2);
    }
    
    #[test]
    fn test_safe_arithmetic() {
        let values = vec![
            FixedPoint::from_int(10),
            FixedPoint::from_int(20),
            FixedPoint::from_int(30),
        ];
        
        assert_eq!(FixedPoint::sum_checked(&values).unwrap().to_int(), 60);
        assert_eq!(FixedPoint::min_of(&values).unwrap().to_int(), 10);
        assert_eq!(FixedPoint::max_of(&values).unwrap().to_int(), 30);
    }
    
    #[test]
    fn test_precision() {
        // Test decimal adjustment
        assert_eq!(precision::adjust_decimals(100, 2, 4).unwrap(), 10000);
        assert_eq!(precision::adjust_decimals(10000, 4, 2).unwrap(), 100);
        
        // Test rounding
        let value = FixedPoint::from_int(3); // 3.0
        let half = FixedPoint(value.0 + (1u128 << 63)); // 3.5
        let rounded = precision::round_to_decimals(half, 0);
        assert_eq!(rounded.to_int(), 4); // 3.5 rounds to 4
    }
}