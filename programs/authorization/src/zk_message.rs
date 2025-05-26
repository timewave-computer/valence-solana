// ZK Message types for Valence Protocol
// Defines structures for ZK-verified cross-chain messages

use anchor_lang::prelude::*;
use anchor_lang::solana_program::hash::Hasher;
use valence_utils::{CompactSerialize, DataPacker, VarInt, TransactionSizeOptimizer};

/// ZK Message structure for cross-chain communication
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ZKMessage {
    /// Registry ID for the ZK program
    pub registry_id: u64,
    /// Sequence number for ordering
    pub sequence: u64,
    /// Source chain identifier
    pub source_chain: u32,
    /// Target chain identifier  
    pub target_chain: u32,
    /// Message payload
    pub payload: Vec<u8>,
    /// Nonce for replay protection
    pub nonce: u64,
    /// Timestamp when message was created
    pub timestamp: i64,
    /// Hash of the message for verification
    pub message_hash: [u8; 32],
}

impl CompactSerialize for ZKMessage {
    fn serialize_compact(&self) -> Result<Vec<u8>> {
        let mut result = Vec::new();
        
        // Use optimized variable-length encoding
        result.extend_from_slice(&VarInt::encode_u64(self.registry_id));
        result.extend_from_slice(&VarInt::encode_u64(self.sequence));
        
        // Pack chain IDs into a single u64 for efficiency
        let chain_ids = DataPacker::pack_u32_array(&[self.source_chain, self.target_chain])?;
        result.extend_from_slice(&chain_ids.to_le_bytes());
        
        // Optimized payload encoding with size validation
        if self.payload.len() > u16::MAX as usize {
            return Err(ProgramError::InvalidInstructionData.into());
        }
        result.extend_from_slice(&(self.payload.len() as u16).to_le_bytes());
        result.extend_from_slice(&self.payload);
        
        // Pack nonce and timestamp efficiently
        result.extend_from_slice(&VarInt::encode_u64(self.nonce));
        result.extend_from_slice(&VarInt::encode_u64(self.timestamp as u64));
        
        // Message hash (fixed 32 bytes)
        result.extend_from_slice(&self.message_hash);
        
        // Validate final size against transaction limits
        TransactionSizeOptimizer::validate_transaction_size(result.len(), 5, 1)?;
        
        Ok(result)
    }
    
    fn deserialize_compact(data: &[u8]) -> Result<Self> {
        let mut offset = 0;
        
        // Decode registry_id
        let (registry_id, consumed) = VarInt::decode_u64(&data[offset..])?;
        offset += consumed;
        
        // Decode sequence
        let (sequence, consumed) = VarInt::decode_u64(&data[offset..])?;
        offset += consumed;
        
        // Unpack chain IDs
        if data.len() < offset + 8 {
            return Err(ProgramError::InvalidAccountData.into());
        }
        let chain_ids_packed = u64::from_le_bytes([
            data[offset], data[offset+1], data[offset+2], data[offset+3],
            data[offset+4], data[offset+5], data[offset+6], data[offset+7]
        ]);
        offset += 8;
        
        let chain_ids = DataPacker::unpack_u32_array(chain_ids_packed, 2);
        let source_chain = chain_ids[0];
        let target_chain = chain_ids[1];
        
        // Decode payload
        if data.len() < offset + 2 {
            return Err(ProgramError::InvalidAccountData.into());
        }
        let payload_len = u16::from_le_bytes([data[offset], data[offset+1]]) as usize;
        offset += 2;
        
        if data.len() < offset + payload_len {
            return Err(ProgramError::InvalidAccountData.into());
        }
        let payload = data[offset..offset + payload_len].to_vec();
        offset += payload_len;
        
        // Decode nonce
        let (nonce, consumed) = VarInt::decode_u64(&data[offset..])?;
        offset += consumed;
        
        // Decode timestamp
        let (timestamp_u64, consumed) = VarInt::decode_u64(&data[offset..])?;
        offset += consumed;
        let timestamp = timestamp_u64 as i64;
        
        // Decode message hash
        if data.len() < offset + 32 {
            return Err(ProgramError::InvalidAccountData.into());
        }
        let mut message_hash = [0u8; 32];
        message_hash.copy_from_slice(&data[offset..offset + 32]);
        
        Ok(Self {
            registry_id,
            sequence,
            source_chain,
            target_chain,
            payload,
            nonce,
            timestamp,
            message_hash,
        })
    }
    
    fn compact_size(&self) -> usize {
        VarInt::encoded_size_u64(self.registry_id) +
        VarInt::encoded_size_u64(self.sequence) +
        8 + // packed chain IDs
        2 + // payload length
        self.payload.len() +
        VarInt::encoded_size_u64(self.nonce) +
        VarInt::encoded_size_u64(self.timestamp as u64) +
        32 // message_hash
    }
}

impl ZKMessage {
    /// Create a new ZK message
    pub fn new(
        registry_id: u64,
        sequence: u64,
        source_chain: u32,
        target_chain: u32,
        payload: Vec<u8>,
        nonce: u64,
    ) -> Result<Self> {
        let timestamp = Clock::get()?.unix_timestamp;
        
        // Calculate message hash
        let mut hasher = Hasher::default();
        hasher.hash(&registry_id.to_le_bytes());
        hasher.hash(&sequence.to_le_bytes());
        hasher.hash(&source_chain.to_le_bytes());
        hasher.hash(&target_chain.to_le_bytes());
        hasher.hash(&payload);
        hasher.hash(&nonce.to_le_bytes());
        hasher.hash(&timestamp.to_le_bytes());
        
        let message_hash = hasher.result().to_bytes();
        
        Ok(Self {
            registry_id,
            sequence,
            source_chain,
            target_chain,
            payload,
            nonce,
            timestamp,
            message_hash,
        })
    }
    
    /// Verify message hash
    pub fn verify_hash(&self) -> bool {
        let mut hasher = Hasher::default();
        hasher.hash(&self.registry_id.to_le_bytes());
        hasher.hash(&self.sequence.to_le_bytes());
        hasher.hash(&self.source_chain.to_le_bytes());
        hasher.hash(&self.target_chain.to_le_bytes());
        hasher.hash(&self.payload);
        hasher.hash(&self.nonce.to_le_bytes());
        hasher.hash(&self.timestamp.to_le_bytes());
        
        let computed_hash = hasher.result().to_bytes();
        computed_hash == self.message_hash
    }

    /// Legacy encode method - use CompactSerialize trait instead
    #[deprecated(note = "Use CompactSerialize::serialize_compact instead")]
    pub fn encode_compact(&self) -> Result<Vec<u8>> {
        self.serialize_compact()
    }

    /// Legacy decode method - use CompactSerialize trait instead
    #[deprecated(note = "Use CompactSerialize::deserialize_compact instead")]
    pub fn decode_compact(data: &[u8]) -> Result<Self> {
        Self::deserialize_compact(data)
    }

    /// Legacy size method - use CompactSerialize trait instead
    #[deprecated(note = "Use CompactSerialize::compact_size instead")]
    pub fn encoded_size(&self) -> usize {
        self.compact_size()
    }
}

// Legacy varint functions removed - using optimized VarInt from utils crate

/// Batch encoding utilities for ZK messages
pub struct ZKMessageBatchEncoder;

impl ZKMessageBatchEncoder {
    /// Encode multiple ZK messages into a single compact batch
    pub fn encode_batch(messages: &[ZKMessage]) -> Result<Vec<u8>> {
        let mut result = Vec::new();
        
        // Message count (use VarInt for efficiency)
        result.extend_from_slice(&VarInt::encode_u32(messages.len() as u32));
        
        // Encode each message
        for message in messages {
            let message_data = message.serialize_compact()?;
            result.extend_from_slice(&VarInt::encode_u32(message_data.len() as u32));
            result.extend_from_slice(&message_data);
        }
        
        // Validate batch size against transaction limits
        TransactionSizeOptimizer::validate_transaction_size(result.len(), 10, 1)?;
        
        Ok(result)
    }
    
    /// Decode batch of ZK messages
    pub fn decode_batch(data: &[u8]) -> Result<Vec<ZKMessage>> {
        let mut offset = 0;
        
        // Message count
        let (message_count, consumed) = VarInt::decode_u32(&data[offset..])?;
        offset += consumed;
        
        if message_count > 100 { // Reasonable batch size limit
            return Err(ProgramError::InvalidInstructionData.into());
        }
        
        let mut messages = Vec::with_capacity(message_count as usize);
        
        // Decode each message
        for _ in 0..message_count {
            let (message_len, consumed) = VarInt::decode_u32(&data[offset..])?;
            offset += consumed;
            
            if data.len() < offset + message_len as usize {
                return Err(ProgramError::InvalidAccountData.into());
            }
            
            let message = ZKMessage::deserialize_compact(&data[offset..offset + message_len as usize])?;
            messages.push(message);
            offset += message_len as usize;
        }
        
        Ok(messages)
    }
    
    /// Calculate estimated size for a batch of messages
    pub fn estimate_batch_size(messages: &[ZKMessage]) -> usize {
        let mut total_size = VarInt::encoded_size_u64(messages.len() as u64);
        
        for message in messages {
            let message_size = message.compact_size();
            total_size += VarInt::encoded_size_u64(message_size as u64) + message_size;
        }
        
        total_size
    }
}

/// ZK Message compression utilities
pub struct ZKMessageCompressor;

impl ZKMessageCompressor {
    /// Compress ZK message payload using simple compression
    pub fn compress_payload(payload: &[u8]) -> Vec<u8> {
        // Use run-length encoding for repeated data
        TransactionSizeOptimizer::optimize_instruction_data(payload)
    }
    
    /// Decompress ZK message payload
    pub fn decompress_payload(compressed: &[u8]) -> Result<Vec<u8>> {
        // For now, just return the data as-is
        // In a full implementation, this would reverse the compression
        Ok(compressed.to_vec())
    }
    
    /// Check if payload would benefit from compression
    pub fn should_compress(payload: &[u8]) -> bool {
        // Compress if payload is large and has repeated patterns
        payload.len() > 256 && has_repeated_patterns(payload)
    }
}

/// Check if data has repeated patterns that would benefit from compression
fn has_repeated_patterns(data: &[u8]) -> bool {
    if data.len() < 16 {
        return false;
    }
    
    let mut repeated_bytes = 0;
    let mut current_byte = data[0];
    let mut count = 1;
    
    for &byte in &data[1..] {
        if byte == current_byte {
            count += 1;
        } else {
            if count >= 3 {
                repeated_bytes += count;
            }
            current_byte = byte;
            count = 1;
        }
    }
    
    // If more than 25% of bytes are in repeated patterns, compression is beneficial
    repeated_bytes > data.len() / 4
}

/// ZK Proof data for message verification
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ZKProof {
    /// The proof bytes
    pub proof: Vec<u8>,
    /// Public inputs for the proof
    pub public_inputs: Vec<u8>,
    /// Verification key identifier
    pub verification_key_id: Pubkey,
}

impl CompactSerialize for ZKProof {
    fn serialize_compact(&self) -> Result<Vec<u8>> {
        let mut result = Vec::new();
        
        // Verification key ID (32 bytes)
        result.extend_from_slice(self.verification_key_id.as_ref());
        
        // Proof length and data (use VarInt for length)
        result.extend_from_slice(&VarInt::encode_u32(self.proof.len() as u32));
        result.extend_from_slice(&self.proof);
        
        // Public inputs length and data (use VarInt for length)
        result.extend_from_slice(&VarInt::encode_u32(self.public_inputs.len() as u32));
        result.extend_from_slice(&self.public_inputs);
        
        // Validate size constraints
        if result.len() > 8192 { // 8KB max for proof data
            return Err(ProgramError::InvalidInstructionData.into());
        }
        
        Ok(result)
    }
    
    fn deserialize_compact(data: &[u8]) -> Result<Self> {
        let mut offset = 0;
        
        // Verification key ID
        if data.len() < 32 {
            return Err(ProgramError::InvalidAccountData.into());
        }
        let verification_key_id = Pubkey::try_from(&data[offset..offset + 32])
            .map_err(|_| ProgramError::InvalidAccountData)?;
        offset += 32;
        
        // Proof length and data
        let (proof_len, consumed) = VarInt::decode_u32(&data[offset..])?;
        offset += consumed;
        
        if data.len() < offset + proof_len as usize {
            return Err(ProgramError::InvalidAccountData.into());
        }
        let proof = data[offset..offset + proof_len as usize].to_vec();
        offset += proof_len as usize;
        
        // Public inputs length and data
        let (inputs_len, consumed) = VarInt::decode_u32(&data[offset..])?;
        offset += consumed;
        
        if data.len() < offset + inputs_len as usize {
            return Err(ProgramError::InvalidAccountData.into());
        }
        let public_inputs = data[offset..offset + inputs_len as usize].to_vec();
        
        Ok(Self {
            proof,
            public_inputs,
            verification_key_id,
        })
    }
    
    fn compact_size(&self) -> usize {
        32 + // verification_key_id
        VarInt::encoded_size_u64(self.proof.len() as u64) +
        self.proof.len() +
        VarInt::encoded_size_u64(self.public_inputs.len() as u64) +
        self.public_inputs.len()
    }
}

/// ZK Message with proof for execution
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ZKMessageWithProof {
    /// The ZK message
    pub message: ZKMessage,
    /// The ZK proof
    pub proof: ZKProof,
}

impl CompactSerialize for ZKMessageWithProof {
    fn serialize_compact(&self) -> Result<Vec<u8>> {
        let mut result = Vec::new();
        
        // Serialize message using compact format
        let message_data = self.message.serialize_compact()?;
        result.extend_from_slice(&VarInt::encode_u32(message_data.len() as u32));
        result.extend_from_slice(&message_data);
        
        // Serialize proof using compact format
        let proof_data = self.proof.serialize_compact()?;
        result.extend_from_slice(&VarInt::encode_u32(proof_data.len() as u32));
        result.extend_from_slice(&proof_data);
        
        // Validate total size against transaction limits
        TransactionSizeOptimizer::validate_transaction_size(result.len(), 8, 1)?;
        
        Ok(result)
    }
    
    fn deserialize_compact(data: &[u8]) -> Result<Self> {
        let mut offset = 0;
        
        // Deserialize message
        let (message_len, consumed) = VarInt::decode_u32(&data[offset..])?;
        offset += consumed;
        
        if data.len() < offset + message_len as usize {
            return Err(ProgramError::InvalidAccountData.into());
        }
        let message = ZKMessage::deserialize_compact(&data[offset..offset + message_len as usize])?;
        offset += message_len as usize;
        
        // Deserialize proof
        let (proof_len, consumed) = VarInt::decode_u32(&data[offset..])?;
        offset += consumed;
        
        if data.len() < offset + proof_len as usize {
            return Err(ProgramError::InvalidAccountData.into());
        }
        let proof = ZKProof::deserialize_compact(&data[offset..offset + proof_len as usize])?;
        
        Ok(Self { message, proof })
    }
    
    fn compact_size(&self) -> usize {
        let message_size = self.message.compact_size();
        let proof_size = self.proof.compact_size();
        
        VarInt::encoded_size_u64(message_size as u64) +
        message_size +
        VarInt::encoded_size_u64(proof_size as u64) +
        proof_size
    }
}

/// Replay protection state
#[account]
pub struct ReplayProtection {
    /// Message hash that has been processed
    pub message_hash: [u8; 32],
    /// Sequence number
    pub sequence: u64,
    /// Timestamp when processed
    pub processed_at: i64,
    /// Bump seed for PDA
    pub bump: u8,
}

/// ZK Message execution state
#[account]
pub struct ZKMessageExecution {
    /// Message hash
    pub message_hash: [u8; 32],
    /// Execution ID
    pub execution_id: u64,
    /// Whether execution was successful
    pub success: bool,
    /// Error data if execution failed
    pub error_data: Option<Vec<u8>>,
    /// Timestamp when executed
    pub executed_at: i64,
    /// Bump seed for PDA
    pub bump: u8,
} 