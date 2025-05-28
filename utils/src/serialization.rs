// Account serialization optimization utilities for minimizing transaction size
use anchor_lang::prelude::*;


/// Maximum transaction size limit in Solana
pub const MAX_TRANSACTION_SIZE: usize = 1_232;

/// Serialization optimization utilities
pub struct SerializationOptimizer;

impl SerializationOptimizer {
    /// Calculate serialized size of an anchor-serializable type
    pub fn calculate_size<T: AnchorSerialize>(data: &T) -> Result<usize> {
        let serialized = data.try_to_vec()
            .map_err(|_| ProgramError::InvalidAccountData)?;
        Ok(serialized.len())
    }
    
    /// Validate that serialized data fits within transaction limits
    pub fn validate_transaction_size(
        instruction_data_size: usize,
        account_count: usize,
    ) -> Result<()> {
        // Base transaction overhead (signatures, headers, etc.)
        let base_overhead = 64; // Approximate base size
        
        // Account metadata overhead (32 bytes pubkey + flags per account)
        let account_overhead = account_count * 34;
        
        // Total estimated transaction size
        let total_size = base_overhead + account_overhead + instruction_data_size;
        
        if total_size > MAX_TRANSACTION_SIZE {
            msg!(
                "Transaction size ({} bytes) exceeds limit ({} bytes)",
                total_size,
                MAX_TRANSACTION_SIZE
            );
            return Err(ProgramError::InvalidInstructionData.into());
        }
        
        Ok(())
    }
    
    /// Optimize string serialization by using compact encoding
    pub fn optimize_string_serialization(s: &str) -> Vec<u8> {
        // Use length-prefixed encoding for strings
        let mut result = Vec::new();
        let len = s.len() as u16; // Use u16 for length to save space vs u32
        result.extend_from_slice(&len.to_le_bytes());
        result.extend_from_slice(s.as_bytes());
        result
    }
    
    /// Optimize vector serialization by using compact length encoding
    pub fn optimize_vec_serialization<T: AnchorSerialize>(vec: &[T]) -> Result<Vec<u8>> {
        let mut result = Vec::new();
        
        // Use u16 for length if possible, otherwise u32
        if vec.len() <= u16::MAX as usize {
            let len = vec.len() as u16;
            result.extend_from_slice(&len.to_le_bytes());
        } else {
            // Use a marker byte to indicate u32 length follows
            result.push(0xFF);
            let len = vec.len() as u32;
            result.extend_from_slice(&len.to_le_bytes());
        }
        
        // Serialize each element
        for item in vec {
            let serialized = item.try_to_vec()
                .map_err(|_| ProgramError::InvalidAccountData)?;
            result.extend_from_slice(&serialized);
        }
        
        Ok(result)
    }
    
    /// Compress boolean flags into bit fields
    pub fn pack_boolean_flags(flags: &[bool]) -> Vec<u8> {
        let mut result = Vec::new();
        let mut current_byte = 0u8;
        let mut bit_position = 0;
        
        for &flag in flags {
            if flag {
                current_byte |= 1 << bit_position;
            }
            
            bit_position += 1;
            
            if bit_position == 8 {
                result.push(current_byte);
                current_byte = 0;
                bit_position = 0;
            }
        }
        
        // Push the last byte if there are remaining bits
        if bit_position > 0 {
            result.push(current_byte);
        }
        
        result
    }
    
    /// Unpack boolean flags from bit fields
    pub fn unpack_boolean_flags(data: &[u8], flag_count: usize) -> Vec<bool> {
        let mut flags = Vec::with_capacity(flag_count);
        
        for (byte_index, &byte) in data.iter().enumerate() {
            for bit_position in 0..8 {
                let flag_index = byte_index * 8 + bit_position;
                if flag_index >= flag_count {
                    break;
                }
                
                let flag = (byte & (1 << bit_position)) != 0;
                flags.push(flag);
            }
        }
        
        flags
    }
}

/// Compact serialization trait for custom types
pub trait CompactSerialize {
    /// Serialize to a compact byte representation
    fn serialize_compact(&self) -> Result<Vec<u8>>;
    
    /// Deserialize from a compact byte representation
    fn deserialize_compact(data: &[u8]) -> Result<Self>
    where
        Self: Sized;
    
    /// Calculate the compact serialized size
    fn compact_size(&self) -> usize;
}

/// Account size optimization utilities
pub struct AccountSizeOptimizer;

impl AccountSizeOptimizer {
    /// Calculate optimal account size with padding
    pub fn calculate_optimal_size(
        base_size: usize,
        dynamic_data_size: usize,
        growth_factor: f32,
    ) -> usize {
        let total_size = base_size + dynamic_data_size;
        let padded_size = (total_size as f32 * (1.0 + growth_factor)) as usize;
        
        // Align to 8-byte boundary for better performance
        (padded_size + 7) & !7
    }
    
    /// Validate account size against rent exemption requirements
    pub fn validate_rent_exemption(account_size: usize, rent: &Rent) -> Result<u64> {
        let minimum_balance = rent.minimum_balance(account_size);
        Ok(minimum_balance)
    }
    
    /// Calculate space requirements for dynamic string fields
    pub fn calculate_string_space(max_length: usize) -> usize {
        // 4 bytes for length prefix + actual string bytes
        4 + max_length
    }
    
    /// Calculate space requirements for dynamic vector fields
    pub fn calculate_vec_space<T>(max_items: usize, item_size: usize) -> usize {
        // 4 bytes for length prefix + items
        4 + (max_items * item_size)
    }
}

/// Instruction data optimization utilities
pub struct InstructionDataOptimizer;

impl InstructionDataOptimizer {
    /// Optimize instruction data by removing unnecessary padding
    pub fn optimize_instruction_data<T: AnchorSerialize>(data: &T) -> Result<Vec<u8>> {
        let mut serialized = data.try_to_vec()
            .map_err(|_| ProgramError::InvalidInstructionData)?;
        
        // Remove trailing zeros (padding)
        while serialized.last() == Some(&0) && serialized.len() > 1 {
            serialized.pop();
        }
        
        Ok(serialized)
    }
    
    /// Batch multiple small instructions into a single instruction
    pub fn batch_instructions<T: AnchorSerialize>(instructions: &[T]) -> Result<Vec<u8>> {
        let mut result = Vec::new();
        
        // Add instruction count
        let count = instructions.len() as u16;
        result.extend_from_slice(&count.to_le_bytes());
        
        // Serialize each instruction
        for instruction in instructions {
            let serialized = instruction.try_to_vec()
                .map_err(|_| ProgramError::InvalidInstructionData)?;
            
            // Add length prefix for each instruction
            let len = serialized.len() as u16;
            result.extend_from_slice(&len.to_le_bytes());
            result.extend_from_slice(&serialized);
        }
        
        Ok(result)
    }
    
    /// Compress repeated data using run-length encoding
    pub fn compress_repeated_data(data: &[u8]) -> Vec<u8> {
        if data.is_empty() {
            return Vec::new();
        }
        
        let mut result = Vec::new();
        let mut current_byte = data[0];
        let mut count = 1u8;
        
        for &byte in &data[1..] {
            if byte == current_byte && count < 255 {
                count += 1;
            } else {
                result.push(count);
                result.push(current_byte);
                current_byte = byte;
                count = 1;
            }
        }
        
        // Add the last run
        result.push(count);
        result.push(current_byte);
        
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_boolean_flag_packing() {
        let flags = vec![true, false, true, true, false, false, true, false, true];
        let packed = SerializationOptimizer::pack_boolean_flags(&flags);
        let unpacked = SerializationOptimizer::unpack_boolean_flags(&packed, flags.len());
        
        assert_eq!(flags, unpacked);
    }
    
    #[test]
    fn test_transaction_size_validation() {
        // Should pass for small transaction
        assert!(SerializationOptimizer::validate_transaction_size(100, 5).is_ok());
        
        // Should fail for large transaction
        assert!(SerializationOptimizer::validate_transaction_size(1200, 10).is_err());
    }
    
    #[test]
    fn test_optimal_account_size() {
        let size = AccountSizeOptimizer::calculate_optimal_size(100, 50, 0.2);
        assert_eq!(size, 184); // (150 * 1.2) rounded up to 8-byte boundary
    }
} 