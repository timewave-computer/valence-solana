use anchor_lang::prelude::*;
use valence_utils::{AccountSizeOptimizer, CompactSerialize};

/// Permission type for an authorization
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum PermissionType {
    /// Anyone can use this authorization
    Public,
    /// Only the owner can use this authorization
    OwnerOnly,
    /// Only specified users can use this authorization
    Allowlist,
}

impl Default for PermissionType {
    fn default() -> Self {
        PermissionType::OwnerOnly
    }
}

/// Priority level for messages
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum Priority {
    /// Low priority, processed last
    Low,
    /// Medium priority, processed after high
    Medium,
    /// High priority, processed first
    High,
}

impl Default for Priority {
    fn default() -> Self {
        Priority::Medium
    }
}

/// Subroutine execution type
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum SubroutineType {
    /// Atomic execution - all messages must succeed
    Atomic,
    /// Non-atomic execution - messages can fail individually
    NonAtomic,
}

impl Default for SubroutineType {
    fn default() -> Self {
        SubroutineType::Atomic
    }
}

/// Result of execution
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq, Debug)]
pub enum ExecutionResult {
    /// Execution succeeded
    Success,
    /// Execution failed
    Failure,
}

impl Default for ExecutionResult {
    fn default() -> Self {
        ExecutionResult::Success
    }
}

/// Message to be processed
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ProcessorMessage {
    /// Program ID to call
    pub program_id: Pubkey,
    /// Instruction data
    pub data: Vec<u8>,
    /// Account metas
    pub accounts: Vec<AccountMetaData>,
}

impl CompactSerialize for ProcessorMessage {
    fn serialize_compact(&self) -> Result<Vec<u8>> {
        let mut result = Vec::new();
        
        // Program ID (32 bytes)
        result.extend_from_slice(self.program_id.as_ref());
        
        // Data length and data (use u16 for length if possible)
        if self.data.len() <= u16::MAX as usize {
            let len = self.data.len() as u16;
            result.extend_from_slice(&len.to_le_bytes());
        } else {
            result.push(0xFF); // Marker for u32 length
            result.push(0xFF);
            let len = self.data.len() as u32;
            result.extend_from_slice(&len.to_le_bytes());
        }
        result.extend_from_slice(&self.data);
        
        // Accounts count and accounts (use u8 for count if possible)
        if self.accounts.len() <= u8::MAX as usize {
            let count = self.accounts.len() as u8;
            result.push(count);
        } else {
            result.push(0xFF); // Marker for u16 count
            let count = self.accounts.len() as u16;
            result.extend_from_slice(&count.to_le_bytes());
        }
        
        // Serialize accounts using compact format
        for account in &self.accounts {
            let compact_account = account.serialize_compact()?;
            result.extend_from_slice(&compact_account);
        }
        
        Ok(result)
    }
    
    fn deserialize_compact(data: &[u8]) -> Result<Self> {
        let mut offset = 0;
        
        // Program ID
        if data.len() < 32 {
            return Err(ProgramError::InvalidAccountData.into());
        }
        let program_id = Pubkey::try_from(&data[offset..offset + 32])
            .map_err(|_| ProgramError::InvalidAccountData)?;
        offset += 32;
        
        // Data length and data
        if data.len() < offset + 2 {
            return Err(ProgramError::InvalidAccountData.into());
        }
        
        let data_len = if data[offset] == 0xFF && data[offset + 1] == 0xFF {
            // u32 length
            offset += 2;
            if data.len() < offset + 4 {
                return Err(ProgramError::InvalidAccountData.into());
            }
            let len = u32::from_le_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]]);
            offset += 4;
            len as usize
        } else {
            // u16 length
            let len = u16::from_le_bytes([data[offset], data[offset + 1]]);
            offset += 2;
            len as usize
        };
        
        if data.len() < offset + data_len {
            return Err(ProgramError::InvalidAccountData.into());
        }
        let instruction_data = data[offset..offset + data_len].to_vec();
        offset += data_len;
        
        // Accounts count
        if data.len() < offset + 1 {
            return Err(ProgramError::InvalidAccountData.into());
        }
        
        let accounts_count = if data[offset] == 0xFF {
            // u16 count
            offset += 1;
            if data.len() < offset + 2 {
                return Err(ProgramError::InvalidAccountData.into());
            }
            let count = u16::from_le_bytes([data[offset], data[offset + 1]]);
            offset += 2;
            count as usize
        } else {
            // u8 count
            let count = data[offset];
            offset += 1;
            count as usize
        };
        
        // Deserialize accounts
        let mut accounts = Vec::with_capacity(accounts_count);
        for _ in 0..accounts_count {
            if data.len() < offset + 33 {
                return Err(ProgramError::InvalidAccountData.into());
            }
            let account = AccountMetaData::deserialize_compact(&data[offset..offset + 33])?;
            accounts.push(account);
            offset += 33;
        }
        
        Ok(ProcessorMessage {
            program_id,
            data: instruction_data,
            accounts,
        })
    }
    
    fn compact_size(&self) -> usize {
        let mut size = 32; // program_id
        
        // Data length encoding
        if self.data.len() <= u16::MAX as usize {
            size += 2; // u16 length
        } else {
            size += 6; // marker + u32 length
        }
        size += self.data.len(); // actual data
        
        // Accounts count encoding
        if self.accounts.len() <= u8::MAX as usize {
            size += 1; // u8 count
        } else {
            size += 3; // marker + u16 count
        }
        size += self.accounts.len() * 33; // compact accounts
        
        size
    }
}

/// Account meta data for cross-program invocations
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct AccountMetaData {
    /// Account pubkey
    pub pubkey: Pubkey,
    /// Is signer
    pub is_signer: bool,
    /// Is writable
    pub is_writable: bool,
}

impl CompactSerialize for AccountMetaData {
    fn serialize_compact(&self) -> Result<Vec<u8>> {
        let mut result = Vec::with_capacity(34);
        result.extend_from_slice(self.pubkey.as_ref());
        
        // Pack boolean flags into a single byte
        let flags = if self.is_signer { 1u8 } else { 0u8 } |
                   if self.is_writable { 2u8 } else { 0u8 };
        result.push(flags);
        
        Ok(result)
    }
    
    fn deserialize_compact(data: &[u8]) -> Result<Self> {
        if data.len() != 33 {
            return Err(ProgramError::InvalidAccountData.into());
        }
        
        let pubkey = Pubkey::try_from(&data[0..32])
            .map_err(|_| ProgramError::InvalidAccountData)?;
        
        let flags = data[32];
        let is_signer = (flags & 1) != 0;
        let is_writable = (flags & 2) != 0;
        
        Ok(AccountMetaData {
            pubkey,
            is_signer,
            is_writable,
        })
    }
    
    fn compact_size(&self) -> usize {
        33 // 32 bytes pubkey + 1 byte flags
    }
}

/// Main authorization state account
#[account]
pub struct AuthorizationState {
    /// Owner of the authorization system
    pub owner: Pubkey,
    /// Sub-owners who can create authorizations
    pub sub_owners: Vec<Pubkey>,
    /// Processor program ID
    pub processor_id: Pubkey,
    /// Registry program ID
    pub registry_id: Pubkey,
    /// Counter for execution IDs
    pub execution_counter: u64,
    /// Bump seed for PDA derivation
    pub bump: u8,
    /// Last processed ZK message sequence number
    pub last_zk_sequence: u64,
    /// ZK message sequence counter for outgoing messages
    pub zk_sequence_counter: u64,
    /// Reserved space for future use
    pub reserved: [u8; 64],
}

impl AuthorizationState {
    /// Calculate space needed for this account
    pub const fn space(sub_owners_count: usize) -> usize {
        8 + // discriminator
        32 + // owner
        4 + (sub_owners_count * 32) + // sub_owners vec
        32 + // processor_id
        32 + // registry_id
        8 + // execution_counter
        1 + // bump
        8 + // last_zk_sequence
        8 + // zk_sequence_counter
        64 // reserved
    }
}

/// Authorization
#[account]
#[derive(Default)]
pub struct Authorization {
    /// Unique identifier (using String for better ergonomics)
    pub label: String,
    /// Owner of this authorization
    pub owner: Pubkey,
    /// Whether authorization is active
    pub is_active: bool,
    /// Who can use this authorization
    pub permission_type: PermissionType,
    /// If permission type is allowlist
    pub allowed_users: Vec<Pubkey>,
    /// Earliest valid timestamp
    pub not_before: i64,
    /// Expiration timestamp
    pub expiration: Option<i64>,
    /// Concurrent execution limit
    pub max_concurrent_executions: u32,
    /// Message priority level
    pub priority: Priority,
    /// Atomic or NonAtomic execution
    pub subroutine_type: SubroutineType,
    /// Current number of in-flight executions
    pub current_executions: u32,
    /// Bump seed for PDA
    pub bump: u8,
}

impl Authorization {
    /// Calculate space needed for this account with optimization
    pub fn space(label_len: usize, allowed_users_count: usize) -> usize {
        let base_size = 8 + // discriminator
            4 + label_len + // label string
            32 + // owner
            1 + // is_active
            1 + // permission_type enum
            4 + (allowed_users_count * 32) + // allowed_users vec
            8 + // not_before
            1 + 8 + // expiration option
            4 + // max_concurrent_executions
            1 + // priority enum
            1 + // subroutine_type enum
            4 + // current_executions
            1; // bump
        
        // Use optimized account size calculation with 20% growth factor
        AccountSizeOptimizer::calculate_optimal_size(base_size, 0, 0.2)
    }
    
    /// Calculate space needed for this account (legacy method)
    pub fn space_legacy(label_len: usize, allowed_users_count: usize) -> usize {
        8 + // discriminator
        4 + label_len + // label string
        32 + // owner
        1 + // is_active
        1 + // permission_type enum
        4 + (allowed_users_count * 32) + // allowed_users vec
        8 + // not_before
        1 + 8 + // expiration option
        4 + // max_concurrent_executions
        1 + // priority enum
        1 + // subroutine_type enum
        4 + // current_executions
        1 // bump
    }
    
    /// Get the label of this authorization
    pub fn get_label(&self) -> &str {
        &self.label
    }
    
    /// Set the label of this authorization
    pub fn set_label(&mut self, label: &str) {
        self.label = label.to_string();
    }
}

/// Current execution tracking
#[account]
#[derive(Default)]
pub struct CurrentExecution {
    /// Unique execution ID
    pub id: u64,
    /// Related authorization (using String for better ergonomics)
    pub authorization_label: String,
    /// Transaction initiator
    pub sender: Pubkey,
    /// Start timestamp
    pub start_time: i64,
    /// Bump seed for PDA
    pub bump: u8,
}

impl CurrentExecution {
    /// Calculate space needed for this account
    pub fn space(label_len: usize) -> usize {
        8 + // discriminator
        8 + // id
        4 + label_len + // authorization_label string
        32 + // sender
        8 + // start_time
        1 // bump
    }
    
    /// Set the authorization label for this execution
    pub fn set_authorization_label(&mut self, label: &str) {
        self.authorization_label = label.to_string();
    }
}

/// ZK Program type classification
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub enum ZKProgramType {
    /// SP1 proof system program
    SP1,
    /// Groth16 proof system program
    Groth16,
    /// PLONK proof system program
    PLONK,
    /// SMT (Sparse Merkle Tree) program
    SMT,
    /// Custom ZK program
    Custom,
}

impl Default for ZKProgramType {
    fn default() -> Self {
        ZKProgramType::Custom
    }
}

/// ZK Registry entry for managing ZK programs
#[account]
pub struct ZKRegistry {
    /// The ZK program ID
    pub program_id: Pubkey,
    /// Associated verification key ID
    pub verification_key_id: Pubkey,
    /// Type of ZK program
    pub program_type: ZKProgramType,
    /// Whether the program is active
    pub is_active: bool,
    /// Timestamp when registered
    pub registered_at: i64,
    /// Last time this program was used
    pub last_used: i64,
    /// Number of times this program has been used
    pub usage_count: u64,
    /// Bump seed for PDA
    pub bump: u8,
}

impl ZKRegistry {
    /// Calculate space needed for this account
    pub const fn space() -> usize {
        8 + // discriminator
        32 + // program_id
        32 + // verification_key_id
        1 + // program_type enum
        1 + // is_active
        8 + // registered_at
        8 + // last_used
        8 + // usage_count
        1 // bump
    }
}

/// ZK Registry entry data structure for return values
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ZKRegistryEntry {
    /// The ZK program ID
    pub program_id: Pubkey,
    /// Associated verification key ID
    pub verification_key_id: Pubkey,
    /// Type of ZK program
    pub program_type: ZKProgramType,
    /// Whether the program is active
    pub is_active: bool,
    /// Timestamp when registered
    pub registered_at: i64,
    /// Last time this program was used
    pub last_used: i64,
    /// Number of times this program has been used
    pub usage_count: u64,
    /// Bump seed for PDA
    pub bump: u8,
} 