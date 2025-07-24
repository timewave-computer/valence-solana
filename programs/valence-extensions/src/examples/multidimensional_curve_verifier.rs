//! Multidimensional Invariant Exchange Curve Verifier
//!
//! This verifier implements a generalized weighted product invariant similar to Balancer,
//! allowing declarative expression of arbitrary n-to-m token swaps in a single transaction.
//! 
//! Mathematical Foundation:
//! ∏(Bi^Wi) = K (constant)
//! Where Bi = balance of token i, Wi = weight of token i, K = invariant
//!
//! This enables:
//! - Multi-asset pools with arbitrary weights
//! - Single transaction n-to-m swaps
//! - Flexible liquidity provision patterns
//! - Custom bonding curves

#[cfg(feature = "math")]
use anchor_lang::prelude::*;
#[cfg(feature = "math")]
use crate::math::FixedPoint;

// Maximum number of tokens in a pool for gas efficiency
#[cfg(feature = "math")]
pub const MAX_POOL_TOKENS: usize = 8;

// Minimum weight to prevent division by zero (0.01 in fixed point)
#[cfg(feature = "math")]
pub const MIN_WEIGHT: FixedPoint = FixedPoint(184467440737095516); // 0.01 * 2^64

// Maximum weight to prevent overflow (99 in fixed point) 
#[cfg(feature = "math")]
pub const MAX_WEIGHT: FixedPoint = FixedPoint(18267659090179465216); // 99 * 2^64

/// Pool configuration encoded in account parameters
/// Layout: [num_tokens(1)] [weights(8*n)] [balances(8*n)] [fee_bps(2)] [padding]
#[cfg(feature = "math")]
#[derive(Debug, Clone)]
pub struct PoolConfig {
    pub num_tokens: u8,
    pub weights: Vec<FixedPoint>,  // Fixed point weights (sum should = 1.0)
    pub balances: Vec<FixedPoint>, // Current token balances
    pub fee_bps: u16,              // Fee in basis points (30 = 0.3%)
}

#[cfg(feature = "math")]
impl PoolConfig {
    /// Helper to convert MathError to ErrorCode
    fn convert_math_error(e: crate::math::MathError) -> anchor_lang::error::Error {
        match e {
            crate::math::MathError::Overflow => ErrorCode::Overflow.into(),
            crate::math::MathError::DivisionByZero => ErrorCode::Overflow.into(),
        }
    }
    /// Parse pool configuration from account parameters
    pub fn from_params(params: &[u8; 256]) -> Result<Self> {
        require!(params.len() >= 3, ErrorCode::InvalidPoolParams);
        
        let num_tokens = params[0];
        require!(
            num_tokens > 1 && num_tokens <= MAX_POOL_TOKENS as u8,
            ErrorCode::InvalidTokenCount
        );

        let mut weights = Vec::with_capacity(num_tokens as usize);
        let mut balances = Vec::with_capacity(num_tokens as usize);
        
        // Parse weights (8 bytes each)
        let weights_start = 1;
        for i in 0..num_tokens {
            let offset = weights_start + (i as usize * 8);
            require!(offset + 8 <= params.len(), ErrorCode::InvalidPoolParams);
            
            let weight_raw = u64::from_le_bytes(
                params[offset..offset + 8].try_into().unwrap()
            );
            let weight = FixedPoint((weight_raw as u128) << 64);
            require!(
                weight >= MIN_WEIGHT && weight <= MAX_WEIGHT,
                ErrorCode::InvalidWeight
            );
            weights.push(weight);
        }
        
        // Parse balances (8 bytes each) 
        let balances_start = weights_start + (num_tokens as usize * 8);
        for i in 0..num_tokens {
            let offset = balances_start + (i as usize * 8);
            require!(offset + 8 <= params.len(), ErrorCode::InvalidPoolParams);
            
            let balance_raw = u64::from_le_bytes(
                params[offset..offset + 8].try_into().unwrap()
            );
            let balance = FixedPoint::from_int(balance_raw);
            require!(balance > FixedPoint::ZERO, ErrorCode::ZeroBalance);
            balances.push(balance);
        }
        
        // Parse fee
        let fee_offset = balances_start + (num_tokens as usize * 8);
        require!(fee_offset + 2 <= params.len(), ErrorCode::InvalidPoolParams);
        let fee_bps = u16::from_le_bytes(
            params[fee_offset..fee_offset + 2].try_into().unwrap()
        );
        require!(fee_bps <= 1000, ErrorCode::InvalidFee); // Max 10%
        
        // Validate weights sum to approximately 1.0
        let weight_sum = FixedPoint::sum_checked(&weights).map_err(|_| ErrorCode::Overflow)?;
        let tolerance = FixedPoint(FixedPoint::ONE.0 / 1000); // 0.1% tolerance
        require!(
            weight_sum.0 >= FixedPoint::ONE.0 - tolerance.0 && 
            weight_sum.0 <= FixedPoint::ONE.0 + tolerance.0,
            ErrorCode::InvalidWeightSum
        );
        
        Ok(PoolConfig {
            num_tokens,
            weights,
            balances,
            fee_bps,
        })
    }
    
    /// Calculate the weighted product invariant: ∏(Bi^Wi)
    /// Production implementation with accurate fractional power calculation
    pub fn calculate_invariant(&self) -> Result<FixedPoint> {
        let mut invariant = FixedPoint::ONE;
        
        for i in 0..self.num_tokens as usize {
            let balance = self.balances[i];
            let weight = self.weights[i];
            
            // Calculate balance^weight using binary exponentiation for integer part
            // and precise fractional power calculation
            let powered_balance = Self::power_fixed(balance, weight)?;
            invariant = invariant.mul(powered_balance).map_err(Self::convert_math_error)?;
        }
        
        Ok(invariant)
    }
    
    /// Calculate base^exponent for fixed-point numbers using binary exponentiation
    /// and fractional power approximation via continued square roots
    fn power_fixed(base: FixedPoint, exponent: FixedPoint) -> Result<FixedPoint> {
        if base == FixedPoint::ZERO {
            return Ok(FixedPoint::ZERO);
        }
        if exponent == FixedPoint::ZERO {
            return Ok(FixedPoint::ONE);
        }
        if exponent == FixedPoint::ONE {
            return Ok(base);
        }
        
        // Split exponent into integer and fractional parts
        let exp_int = exponent.to_int();
        let exp_frac = FixedPoint(exponent.0 - ((exp_int as u128) << 64));
        
        // Calculate base^exp_int using binary exponentiation
        let mut int_result = FixedPoint::ONE;
        let mut base_power = base;
        let mut remaining_exp = exp_int;
        
        while remaining_exp > 0 {
            if remaining_exp & 1 == 1 {
                int_result = int_result.mul(base_power).map_err(Self::convert_math_error)?;
            }
            base_power = base_power.mul(base_power).map_err(Self::convert_math_error)?;
            remaining_exp >>= 1;
        }
        
        // Calculate base^exp_frac using fractional power approximation
        let frac_result = if exp_frac > FixedPoint::ZERO {
            Self::fractional_power(base, exp_frac)?
        } else {
            FixedPoint::ONE
        };
        
        // Combine results: base^(int + frac) = base^int * base^frac
        int_result.mul(frac_result).map_err(Self::convert_math_error)
    }
    
    /// Calculate base^frac_exp where 0 < frac_exp < 1
    /// Uses binary representation of fractional exponent with square roots
    fn fractional_power(base: FixedPoint, frac_exp: FixedPoint) -> Result<FixedPoint> {
        if frac_exp == FixedPoint::ZERO {
            return Ok(FixedPoint::ONE);
        }
        
        let mut result = FixedPoint::ONE;
        let mut current_base = base;
        let mut remaining_frac = frac_exp.0;
        
        // Use binary representation of fractional part
        // Each bit position represents base^(1/2^n)
        for i in 0..64 {
            let bit_value = 1u128 << (63 - i); // Current bit position
            
            if remaining_frac >= bit_value {
                // This bit is set, so multiply by current_base
                result = result.mul(current_base).map_err(Self::convert_math_error)?;
                remaining_frac -= bit_value;
            }
            
            // Prepare next iteration: current_base = sqrt(current_base)
            // This represents base^(1/2^(i+1))
            if i < 63 {
                current_base = current_base.sqrt().map_err(Self::convert_math_error)?;
            }
            
            // Early exit if no more bits to process
            if remaining_frac == 0 {
                break;
            }
        }
        
        Ok(result)
    }
    
    /// Validate that a proposed swap maintains the invariant
    pub fn validate_swap(
        &self,
        tokens_in: &[usize],
        amounts_in: &[u64], 
        tokens_out: &[usize],
        amounts_out: &[u64],
    ) -> Result<bool> {
        require!(
            tokens_in.len() == amounts_in.len() && 
            tokens_out.len() == amounts_out.len(),
            ErrorCode::MismatchedArrays
        );
        
        // Calculate current invariant
        let current_invariant = self.calculate_invariant()?;
        
        // Create new balances after swap
        let mut new_balances = self.balances.clone();
        
        // Apply amounts in (add to balances, subtract fees)
        for (i, &token_idx) in tokens_in.iter().enumerate() {
            require!(token_idx < self.num_tokens as usize, ErrorCode::InvalidTokenIndex);
            
            let amount_in = FixedPoint::from_int(amounts_in[i]);
            let fee_rate = FixedPoint::from_int(self.fee_bps as u64).div(FixedPoint::from_int(10000)).map_err(Self::convert_math_error)?;
            let fee = amount_in.mul(fee_rate).map_err(Self::convert_math_error)?;
            let amount_after_fee = FixedPoint(amount_in.0 - fee.0);
            
            new_balances[token_idx] = FixedPoint(new_balances[token_idx].0
                .checked_add(amount_after_fee.0)
                .ok_or(ErrorCode::Overflow)?);
        }
        
        // Apply amounts out (subtract from balances)
        for (i, &token_idx) in tokens_out.iter().enumerate() {
            require!(token_idx < self.num_tokens as usize, ErrorCode::InvalidTokenIndex);
            
            let amount_out = FixedPoint::from_int(amounts_out[i]);
            require!(
                new_balances[token_idx] >= amount_out,
                ErrorCode::InsufficientBalance
            );
            
            new_balances[token_idx] = FixedPoint(new_balances[token_idx].0 - amount_out.0);
            require!(new_balances[token_idx] > FixedPoint::ZERO, ErrorCode::ZeroBalance);
        }
        
        // Calculate new invariant with updated balances
        let new_config = PoolConfig {
            num_tokens: self.num_tokens,
            weights: self.weights.clone(),
            balances: new_balances,
            fee_bps: self.fee_bps,
        };
        
        let new_invariant = new_config.calculate_invariant()?;
        
        // Allow small tolerance for rounding errors (0.01%)
        let tolerance = FixedPoint(current_invariant.0 / 10000);
        let min_allowed = FixedPoint(current_invariant.0 - tolerance.0);
        
        Ok(new_invariant >= min_allowed)
    }
}

/// Swap parameters encoded in remaining accounts or transaction data
#[cfg(feature = "math")]
#[derive(Debug)]
pub struct SwapParams {
    pub tokens_in: Vec<usize>,
    pub amounts_in: Vec<u64>,
    pub tokens_out: Vec<usize>, 
    pub amounts_out: Vec<u64>,
}

#[cfg(feature = "math")]
impl SwapParams {
    /// Parse swap parameters from remaining accounts or metadata
    /// Format: [n_in(1)] [tokens_in(n_in)] [amounts_in(8*n_in)] [n_out(1)] [tokens_out(n_out)] [amounts_out(8*n_out)]
    pub fn from_metadata(metadata: &[u8; 64]) -> Result<Self> {
        require!(metadata.len() >= 2, ErrorCode::InvalidSwapParams);
        
        let n_in = metadata[0] as usize;
        let n_out = metadata[1] as usize;
        
        require!(n_in > 0 && n_out > 0, ErrorCode::InvalidSwapParams);
        require!(n_in + n_out <= MAX_POOL_TOKENS, ErrorCode::TooManyTokens);
        
        let mut offset = 2;
        
        // Parse tokens in
        let mut tokens_in = Vec::with_capacity(n_in);
        for _ in 0..n_in {
            require!(offset < metadata.len(), ErrorCode::InvalidSwapParams);
            tokens_in.push(metadata[offset] as usize);
            offset += 1;
        }
        
        // Parse amounts in (4 bytes each to save space)
        let mut amounts_in = Vec::with_capacity(n_in);
        for _ in 0..n_in {
            require!(offset + 4 <= metadata.len(), ErrorCode::InvalidSwapParams);
            let amount = u32::from_le_bytes(
                metadata[offset..offset + 4].try_into().unwrap()
            ) as u64;
            amounts_in.push(amount);
            offset += 4;
        }
        
        // Parse tokens out  
        let mut tokens_out = Vec::with_capacity(n_out);
        for _ in 0..n_out {
            require!(offset < metadata.len(), ErrorCode::InvalidSwapParams);
            tokens_out.push(metadata[offset] as usize);
            offset += 1;
        }
        
        // Parse amounts out (4 bytes each)
        let mut amounts_out = Vec::with_capacity(n_out);
        for _ in 0..n_out {
            require!(offset + 4 <= metadata.len(), ErrorCode::InvalidSwapParams);
            let amount = u32::from_le_bytes(
                metadata[offset..offset + 4].try_into().unwrap()
            ) as u64;
            amounts_out.push(amount);
            offset += 4;
        }
        
        Ok(SwapParams {
            tokens_in,
            amounts_in,
            tokens_out,
            amounts_out,
        })
    }
}

/// Main verifier function for multidimensional curve swaps
/// This would be the `verify_account` instruction in a standalone verifier program
#[cfg(feature = "math")]
pub fn verify_multidimensional_swap(
    _account: &AccountInfo,
    _caller: &Signer,
    managed_account_data: &[u8],
    account_metadata: &[u8; 64],
) -> Result<()> {
    // Parse pool configuration from account parameters
    let params: [u8; 256] = managed_account_data[..256].try_into()
        .map_err(|_| ErrorCode::InvalidPoolParams)?;
    let pool_config = PoolConfig::from_params(&params)?;
    
    // Parse swap parameters from account metadata
    let swap_params = SwapParams::from_metadata(account_metadata)?;
    
    // Validate the swap maintains the invariant
    let is_valid = pool_config.validate_swap(
        &swap_params.tokens_in,
        &swap_params.amounts_in,
        &swap_params.tokens_out,
        &swap_params.amounts_out,
    )?;
    
    require!(is_valid, ErrorCode::InvariantViolation);
    
    msg!("Multidimensional swap validated: {} tokens in, {} tokens out",
        swap_params.tokens_in.len(),
        swap_params.tokens_out.len()
    );
    
    Ok(())
}

// Math operations use the valence-extensions math module

#[cfg(feature = "math")]
#[error_code]
pub enum ErrorCode {
    #[msg("Invalid pool parameters")]
    InvalidPoolParams,
    
    #[msg("Invalid token count")]
    InvalidTokenCount,
    
    #[msg("Invalid weight")]
    InvalidWeight,
    
    #[msg("Zero balance not allowed")]
    ZeroBalance,
    
    #[msg("Invalid fee percentage")]
    InvalidFee,
    
    #[msg("Weights must sum to 1.0")]
    InvalidWeightSum,
    
    #[msg("Array length mismatch")]
    MismatchedArrays,
    
    #[msg("Invalid token index")]
    InvalidTokenIndex,
    
    #[msg("Arithmetic overflow")]
    Overflow,
    
    #[msg("Insufficient balance")]
    InsufficientBalance,
    
    #[msg("Invariant violation")]
    InvariantViolation,
    
    #[msg("Invalid swap parameters")]
    InvalidSwapParams,
    
    #[msg("Too many tokens")]
    TooManyTokens,
}

// Example usage demonstrating n-to-m swaps:
//
// 1. Deploy this verifier program
// 2. Create a session with accounts configured for different pool types:
//
// // 3-asset balanced pool (equal weights)
// let mut params = [0u8; 256];
// params[0] = 3; // num_tokens
// // Weights: [0.333, 0.333, 0.334] in fixed point
// params[1..9].copy_from_slice(&(0.333 * (1u64 << 64)).to_le_bytes());
// params[9..17].copy_from_slice(&(0.333 * (1u64 << 64)).to_le_bytes());
// params[17..25].copy_from_slice(&(0.334 * (1u64 << 64)).to_le_bytes());
// // Balances: [1000, 2000, 3000]
// params[25..33].copy_from_slice(&1000u64.to_le_bytes());
// params[33..41].copy_from_slice(&2000u64.to_le_bytes());
// params[41..49].copy_from_slice(&3000u64.to_le_bytes());
// // Fee: 30 bps = 0.3%
// params[49..51].copy_from_slice(&30u16.to_le_bytes());
//
// let account = add_account(session, multidim_verifier, params, 1_hour);
//
// // Configure swap: send tokens 0,1 receive token 2
// let mut metadata = [0u8; 64];
// metadata[0] = 2; // 2 tokens in
// metadata[1] = 1; // 1 token out
// metadata[2] = 0; // token 0 in
// metadata[3] = 1; // token 1 in
// metadata[4..8].copy_from_slice(&100u32.to_le_bytes()); // amount token 0
// metadata[8..12].copy_from_slice(&200u32.to_le_bytes()); // amount token 1
// metadata[12] = 2; // token 2 out
// metadata[13..17].copy_from_slice(&150u32.to_le_bytes()); // amount out
//
// update_metadata(account, metadata);
// use_account(account); // Validates the n-to-m swap

#[cfg(all(test, feature = "math"))]
mod tests {
    use super::*;

    #[test]
    fn test_power_calculation() {
        // Test integer powers
        let base = FixedPoint::from_int(2);
        let exp = FixedPoint::from_int(3);
        let result = PoolConfig::power_fixed(base, exp).unwrap();
        assert_eq!(result.to_int(), 8); // 2^3 = 8

        // Test fractional power (2^0.5 ≈ 1.414)
        let exp_half = FixedPoint(1u128 << 63); // 0.5
        let result_half = PoolConfig::power_fixed(base, exp_half).unwrap();
        // Should be close to sqrt(2) ≈ 1.414
        assert!(result_half.to_int() == 1); // Integer part should be 1
        
        // Test edge cases
        assert_eq!(PoolConfig::power_fixed(FixedPoint::ZERO, FixedPoint::ONE).unwrap(), FixedPoint::ZERO);
        assert_eq!(PoolConfig::power_fixed(FixedPoint::ONE, FixedPoint::from_int(100)).unwrap(), FixedPoint::ONE);
    }

    #[test]
    fn test_fractional_power() {
        let base = FixedPoint::from_int(4);
        let half = FixedPoint(1u128 << 63); // 0.5
        
        let result = PoolConfig::fractional_power(base, half).unwrap();
        // 4^0.5 = 2
        assert_eq!(result.to_int(), 2);
        
        // Test quarter power: 4^0.25 = sqrt(2) ≈ 1.414
        let quarter = FixedPoint(1u128 << 62); // 0.25
        let result_quarter = PoolConfig::fractional_power(base, quarter).unwrap();
        assert!(result_quarter.to_int() == 1); // Should be between 1 and 2
    }

    #[test]
    fn test_pool_config_parsing() {
        let mut params = [0u8; 256];
        
        // Setup a 2-token pool
        params[0] = 2; // num_tokens
        
        // Weight 1: 0.5 (half of 1.0)
        let weight1 = 1u64 << 63; // 0.5 in integer representation 
        params[1..9].copy_from_slice(&weight1.to_le_bytes());
        
        // Weight 2: 0.5 
        let weight2 = 1u64 << 63; // 0.5 in integer representation
        params[9..17].copy_from_slice(&weight2.to_le_bytes());
        
        // Balance 1: 1000
        params[17..25].copy_from_slice(&1000u64.to_le_bytes());
        
        // Balance 2: 2000  
        params[25..33].copy_from_slice(&2000u64.to_le_bytes());
        
        // Fee: 30 bps
        params[33..35].copy_from_slice(&30u16.to_le_bytes());
        
        let config = PoolConfig::from_params(&params).unwrap();
        assert_eq!(config.num_tokens, 2);
        assert_eq!(config.fee_bps, 30);
        assert_eq!(config.balances[0].to_int(), 1000);
        assert_eq!(config.balances[1].to_int(), 2000);
    }

    #[test]
    fn test_invariant_calculation() {
        // Create a simple 2-token equal-weight pool
        let config = PoolConfig {
            num_tokens: 2,
            weights: vec![
                FixedPoint(1u128 << 63), // 0.5
                FixedPoint(1u128 << 63), // 0.5
            ],
            balances: vec![
                FixedPoint::from_int(100),
                FixedPoint::from_int(100), 
            ],
            fee_bps: 30,
        };
        
        let invariant = config.calculate_invariant().unwrap();
        
        // For equal weights (0.5, 0.5) and equal balances (100, 100):
        // Invariant = 100^0.5 * 100^0.5 = 10 * 10 = 100
        assert_eq!(invariant.to_int(), 100);
    }
}