// Data packing optimization utilities for staying under transaction size limits
use anchor_lang::prelude::*;

/// Maximum transaction size limit in Solana
pub const MAX_TRANSACTION_SIZE: usize = 1_232;

/// Estimated overhead for transaction metadata
pub const TRANSACTION_OVERHEAD: usize = 64;

/// Estimated overhead per account in transaction
pub const ACCOUNT_OVERHEAD: usize = 34;

/// Data packing utilities for efficient serialization
pub struct DataPacker;

impl DataPacker {
    /// Pack multiple u8 values into a single u64
    pub fn pack_u8_array(values: &[u8]) -> Result<u64> {
        if values.len() > 8 {
            return Err(ProgramError::InvalidArgument.into());
        }
        
        let mut packed = 0u64;
        for (i, &value) in values.iter().enumerate() {
            packed |= (value as u64) << (i * 8);
        }
        
        Ok(packed)
    }
    
    /// Unpack u64 into array of u8 values
    pub fn unpack_u8_array(packed: u64, count: usize) -> Vec<u8> {
        let mut values = Vec::with_capacity(count);
        for i in 0..count.min(8) {
            let value = ((packed >> (i * 8)) & 0xFF) as u8;
            values.push(value);
        }
        values
    }
    
    /// Pack multiple u16 values into a single u64
    pub fn pack_u16_array(values: &[u16]) -> Result<u64> {
        if values.len() > 4 {
            return Err(ProgramError::InvalidArgument.into());
        }
        
        let mut packed = 0u64;
        for (i, &value) in values.iter().enumerate() {
            packed |= (value as u64) << (i * 16);
        }
        
        Ok(packed)
    }
    
    /// Unpack u64 into array of u16 values
    pub fn unpack_u16_array(packed: u64, count: usize) -> Vec<u16> {
        let mut values = Vec::with_capacity(count);
        for i in 0..count.min(4) {
            let value = ((packed >> (i * 16)) & 0xFFFF) as u16;
            values.push(value);
        }
        values
    }
    
    /// Pack multiple u32 values into a single u64
    pub fn pack_u32_array(values: &[u32]) -> Result<u64> {
        if values.len() > 2 {
            return Err(ProgramError::InvalidArgument.into());
        }
        
        let mut packed = 0u64;
        for (i, &value) in values.iter().enumerate() {
            packed |= (value as u64) << (i * 32);
        }
        
        Ok(packed)
    }
    
    /// Unpack u64 into array of u32 values
    pub fn unpack_u32_array(packed: u64, count: usize) -> Vec<u32> {
        let mut values = Vec::with_capacity(count);
        for i in 0..count.min(2) {
            let value = ((packed >> (i * 32)) & 0xFFFFFFFF) as u32;
            values.push(value);
        }
        values
    }
    
    /// Pack boolean flags into a single byte
    pub fn pack_bool_flags(flags: &[bool]) -> Result<u8> {
        if flags.len() > 8 {
            return Err(ProgramError::InvalidArgument.into());
        }
        
        let mut packed = 0u8;
        for (i, &flag) in flags.iter().enumerate() {
            if flag {
                packed |= 1 << i;
            }
        }
        
        Ok(packed)
    }
    
    /// Unpack byte into boolean flags
    pub fn unpack_bool_flags(packed: u8, count: usize) -> Vec<bool> {
        let mut flags = Vec::with_capacity(count);
        for i in 0..count.min(8) {
            let flag = (packed & (1 << i)) != 0;
            flags.push(flag);
        }
        flags
    }
    
    /// Pack enum values (up to 256 variants) into bytes
    pub fn pack_enum_array<T>(enums: &[T]) -> Vec<u8> 
    where
        T: Into<u8> + Copy,
    {
        enums.iter().map(|&e| e.into()).collect()
    }
    
    /// Calculate packed size for various data types
    pub fn calculate_packed_size(
        u8_arrays: usize,
        u16_arrays: usize,
        u32_arrays: usize,
        bool_flag_groups: usize,
        enum_count: usize,
    ) -> usize {
        (u8_arrays + u16_arrays + u32_arrays) * 8 + // Each array packs into u64
        bool_flag_groups + // Each group packs into u8
        enum_count // Each enum is u8
    }
}

/// Bit field utilities for efficient flag storage
pub struct BitField {
    data: Vec<u8>,
    bit_count: usize,
}

impl BitField {
    /// Create a new bit field with specified capacity
    pub fn new(bit_count: usize) -> Self {
        let byte_count = bit_count.div_ceil(8);
        Self {
            data: vec![0; byte_count],
            bit_count,
        }
    }
    
    /// Set a bit at the specified index
    pub fn set(&mut self, index: usize, value: bool) -> Result<()> {
        if index >= self.bit_count {
            return Err(ProgramError::InvalidArgument.into());
        }
        
        let byte_index = index / 8;
        let bit_index = index % 8;
        
        if value {
            self.data[byte_index] |= 1 << bit_index;
        } else {
            self.data[byte_index] &= !(1 << bit_index);
        }
        
        Ok(())
    }
    
    /// Get a bit at the specified index
    pub fn get(&self, index: usize) -> Result<bool> {
        if index >= self.bit_count {
            return Err(ProgramError::InvalidArgument.into());
        }
        
        let byte_index = index / 8;
        let bit_index = index % 8;
        
        Ok((self.data[byte_index] & (1 << bit_index)) != 0)
    }
    
    /// Get the raw byte data
    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
    
    /// Create from raw byte data
    pub fn from_bytes(data: Vec<u8>, bit_count: usize) -> Self {
        Self { data, bit_count }
    }
    
    /// Get the size in bytes
    pub fn byte_size(&self) -> usize {
        self.data.len()
    }
}

/// Variable-length integer encoding for space efficiency
pub struct VarInt;

impl VarInt {
    /// Encode a u64 as a variable-length integer
    pub fn encode_u64(mut value: u64) -> Vec<u8> {
        let mut result = Vec::new();
        
        while value >= 0x80 {
            result.push((value & 0x7F) as u8 | 0x80);
            value >>= 7;
        }
        result.push(value as u8);
        
        result
    }
    
    /// Decode a variable-length integer to u64
    pub fn decode_u64(data: &[u8]) -> Result<(u64, usize)> {
        let mut value = 0u64;
        let mut shift = 0;
        let mut bytes_read = 0;
        
        for &byte in data {
            bytes_read += 1;
            
            if shift >= 64 {
                return Err(ProgramError::InvalidAccountData.into());
            }
            
            value |= ((byte & 0x7F) as u64) << shift;
            
            if (byte & 0x80) == 0 {
                return Ok((value, bytes_read));
            }
            
            shift += 7;
        }
        
        Err(ProgramError::InvalidAccountData.into())
    }
    
    /// Encode a u32 as a variable-length integer
    pub fn encode_u32(value: u32) -> Vec<u8> {
        Self::encode_u64(value as u64)
    }
    
    /// Decode a variable-length integer to u32
    pub fn decode_u32(data: &[u8]) -> Result<(u32, usize)> {
        let (value, bytes_read) = Self::decode_u64(data)?;
        if value > u32::MAX as u64 {
            return Err(ProgramError::InvalidAccountData.into());
        }
        Ok((value as u32, bytes_read))
    }
    
    /// Calculate the encoded size of a u64
    pub fn encoded_size_u64(value: u64) -> usize {
        if value == 0 {
            return 1;
        }
        
        let mut size = 0;
        let mut v = value;
        while v > 0 {
            size += 1;
            v >>= 7;
        }
        size
    }
}

/// Transaction size optimizer
pub struct TransactionSizeOptimizer;

impl TransactionSizeOptimizer {
    /// Estimate total transaction size
    pub fn estimate_transaction_size(
        instruction_data_size: usize,
        account_count: usize,
        signature_count: usize,
    ) -> usize {
        TRANSACTION_OVERHEAD +
        (signature_count * 64) + // Signatures
        (account_count * ACCOUNT_OVERHEAD) + // Account metadata
        instruction_data_size // Instruction data
    }
    
    /// Check if transaction fits within size limit
    pub fn validate_transaction_size(
        instruction_data_size: usize,
        account_count: usize,
        signature_count: usize,
    ) -> Result<()> {
        let total_size = Self::estimate_transaction_size(
            instruction_data_size,
            account_count,
            signature_count,
        );
        
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
    
    /// Calculate maximum instruction data size for given constraints
    pub fn max_instruction_data_size(
        account_count: usize,
        signature_count: usize,
    ) -> usize {
        let overhead = TRANSACTION_OVERHEAD +
            (signature_count * 64) +
            (account_count * ACCOUNT_OVERHEAD);
        
        MAX_TRANSACTION_SIZE.saturating_sub(overhead)
    }
    
    /// Optimize instruction data by removing padding and compressing
    pub fn optimize_instruction_data(data: &[u8]) -> Vec<u8> {
        // Remove trailing zeros
        let mut optimized = data.to_vec();
        while optimized.last() == Some(&0) && optimized.len() > 1 {
            optimized.pop();
        }
        
        // Apply simple run-length encoding for repeated bytes
        Self::apply_rle_compression(&optimized)
    }
    
    /// Apply run-length encoding compression
    fn apply_rle_compression(data: &[u8]) -> Vec<u8> {
        if data.is_empty() {
            return Vec::new();
        }
        
        let mut compressed = Vec::new();
        let mut i = 0;
        
        while i < data.len() {
            let current_byte = data[i];
            let mut count = 1;
            
            // Count consecutive identical bytes
            while i + count < data.len() && 
                  data[i + count] == current_byte && 
                  count < 255 {
                count += 1;
            }
            
            if count >= 3 {
                // Use RLE for runs of 3 or more
                compressed.push(0xFF); // RLE marker
                compressed.push(count as u8);
                compressed.push(current_byte);
            } else {
                // Copy bytes directly for short runs
                for _ in 0..count {
                    compressed.push(current_byte);
                }
            }
            
            i += count;
        }
        
        // Only return compressed version if it's actually smaller
        if compressed.len() < data.len() {
            compressed
        } else {
            data.to_vec()
        }
    }
}

/// Compact string encoding utilities
pub struct CompactString;

impl CompactString {
    /// Encode string with length prefix (u8 for strings <= 255 chars)
    pub fn encode_short(s: &str) -> Result<Vec<u8>> {
        if s.len() > 255 {
            return Err(ProgramError::InvalidArgument.into());
        }
        
        let mut result = Vec::with_capacity(1 + s.len());
        result.push(s.len() as u8);
        result.extend_from_slice(s.as_bytes());
        
        Ok(result)
    }
    
    /// Decode short string
    pub fn decode_short(data: &[u8]) -> Result<(String, usize)> {
        if data.is_empty() {
            return Err(ProgramError::InvalidAccountData.into());
        }
        
        let len = data[0] as usize;
        if data.len() < 1 + len {
            return Err(ProgramError::InvalidAccountData.into());
        }
        
        let string_data = &data[1..1 + len];
        let s = String::from_utf8(string_data.to_vec())
            .map_err(|_| ProgramError::InvalidAccountData)?;
        
        Ok((s, 1 + len))
    }
    
    /// Encode string with variable-length prefix
    pub fn encode_var(s: &str) -> Vec<u8> {
        let mut result = VarInt::encode_u32(s.len() as u32);
        result.extend_from_slice(s.as_bytes());
        result
    }
    
    /// Decode variable-length string
    pub fn decode_var(data: &[u8]) -> Result<(String, usize)> {
        let (len, len_bytes) = VarInt::decode_u32(data)?;
        let len = len as usize;
        
        if data.len() < len_bytes + len {
            return Err(ProgramError::InvalidAccountData.into());
        }
        
        let string_data = &data[len_bytes..len_bytes + len];
        let s = String::from_utf8(string_data.to_vec())
            .map_err(|_| ProgramError::InvalidAccountData)?;
        
        Ok((s, len_bytes + len))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_u8_packing() {
        let values = vec![1, 2, 3, 4, 5];
        let packed = DataPacker::pack_u8_array(&values).unwrap();
        let unpacked = DataPacker::unpack_u8_array(packed, values.len());
        assert_eq!(values, unpacked);
    }
    
    #[test]
    fn test_bool_packing() {
        let flags = vec![true, false, true, false, true];
        let packed = DataPacker::pack_bool_flags(&flags).unwrap();
        let unpacked = DataPacker::unpack_bool_flags(packed, flags.len());
        assert_eq!(flags, unpacked);
    }
    
    #[test]
    fn test_varint_encoding() {
        let value = 12345u64;
        let encoded = VarInt::encode_u64(value);
        let (decoded, _) = VarInt::decode_u64(&encoded).unwrap();
        assert_eq!(value, decoded);
    }
    
    #[test]
    fn test_transaction_size_validation() {
        // Should pass for small transaction
        assert!(TransactionSizeOptimizer::validate_transaction_size(100, 5, 1).is_ok());
        
        // Should fail for large transaction
        assert!(TransactionSizeOptimizer::validate_transaction_size(1200, 10, 1).is_err());
    }
    
    #[test]
    fn test_compact_string() {
        let s = "Hello, World!";
        let encoded = CompactString::encode_short(s).unwrap();
        let (decoded, _) = CompactString::decode_short(&encoded).unwrap();
        assert_eq!(s, decoded);
    }
} 