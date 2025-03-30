use anchor_lang::prelude::*;

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

/// Authorization Program state
#[account]
pub struct AuthorizationState {
    /// Program owner
    pub owner: Pubkey,
    /// Secondary authorities
    pub sub_owners: Vec<Pubkey>,
    /// Processor program ID
    pub processor_program_id: Pubkey,
    /// Unique ID for executions
    pub execution_counter: u64,
    /// Address of the Valence Registry
    pub valence_registry: Pubkey,
    /// Bump seed for PDA
    pub bump: u8,
}

/// Authorization
#[account]
#[derive(Default)]
pub struct Authorization {
    /// Unique identifier (fixed size char array)
    pub label: [u8; 32],
    /// Label length
    pub label_length: u8,
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
    pub fn set_label(&mut self, label: &str) {
        let bytes = label.as_bytes();
        let len = std::cmp::min(bytes.len(), 32);
        self.label[..len].copy_from_slice(&bytes[..len]);
        self.label_length = len as u8;
    }
    
    pub fn get_label(&self) -> String {
        let len = self.label_length as usize;
        String::from_utf8_lossy(&self.label[..len]).to_string()
    }
}

/// Current execution tracking
#[account]
#[derive(Default)]
pub struct CurrentExecution {
    /// Unique execution ID
    pub id: u64,
    /// Related authorization (fixed size char array)
    pub authorization_label: [u8; 32],
    /// Label length
    pub label_length: u8,
    /// Transaction initiator
    pub sender: Pubkey,
    /// Start timestamp
    pub start_time: i64,
    /// Bump seed for PDA
    pub bump: u8,
}

impl CurrentExecution {
    pub fn set_authorization_label(&mut self, label: &str) {
        let bytes = label.as_bytes();
        let len = std::cmp::min(bytes.len(), 32);
        self.authorization_label[..len].copy_from_slice(&bytes[..len]);
        self.label_length = len as u8;
    }
    
    pub fn get_authorization_label(&self) -> String {
        let len = self.label_length as usize;
        String::from_utf8_lossy(&self.authorization_label[..len]).to_string()
    }
} 