use crate::{Result, ValenceClient, SdkError};
use anchor_lang::prelude::*;
use solana_sdk::instruction::Instruction;
use valence_kernel::{
    state::{CreateSessionParams, RegisteredAccount, RegisteredProgram},
    OperationBatch,
    KernelOperation,
    MAX_BATCH_ACCOUNTS, MAX_BATCH_OPERATIONS, MAX_CPI_ACCOUNT_INDICES, MAX_OPERATION_DATA_SIZE,
    ACCESS_MODE_READ, ACCESS_MODE_WRITE, ACCESS_MODE_READ_WRITE,
};

/// Builder for creating sessions
pub struct SessionBuilder<'a> {
    client: &'a ValenceClient,
    namespace_path: String,
    parent_session: Option<Pubkey>,
    allow_unregistered_cpi: bool,
    initial_borrowable: Vec<RegisteredAccount>,
    initial_programs: Vec<RegisteredProgram>,
    metadata: [u8; 64],
}

impl<'a> SessionBuilder<'a> {
    /// Create a new session builder
    pub fn new(client: &'a ValenceClient, namespace_path: String) -> Self {
        Self {
            client,
            namespace_path,
            parent_session: None,
            allow_unregistered_cpi: false,
            initial_borrowable: Vec::new(),
            initial_programs: Vec::new(),
            metadata: [0u8; 64],
        }
    }

    /// Set the parent session for hierarchical relationships
    pub fn parent_session(mut self, parent: Pubkey) -> Self {
        self.parent_session = Some(parent);
        self
    }

    /// Enable unsafe raw CPI (opt-in to risk)
    pub fn allow_unregistered_cpi(mut self) -> Self {
        self.allow_unregistered_cpi = true;
        self
    }

    /// Add initial borrowable accounts
    pub fn with_borrowable_accounts(mut self, accounts: Vec<RegisteredAccount>) -> Self {
        self.initial_borrowable = accounts;
        self
    }

    /// Add initial program registrations
    pub fn with_programs(mut self, programs: Vec<RegisteredProgram>) -> Self {
        self.initial_programs = programs;
        self
    }

    /// Set metadata
    pub fn metadata(mut self, metadata: [u8; 64]) -> Self {
        self.metadata = metadata;
        self
    }

    /// Build the CreateSessionParams
    pub fn build_params(&self) -> Result<CreateSessionParams> {
        let namespace_bytes = self.namespace_path.as_bytes();
        if namespace_bytes.len() > 255 {
            return Err(SdkError::InvalidSessionConfig);
        }
        
        let mut namespace_path = [0u8; 128];
        namespace_path[..namespace_bytes.len()].copy_from_slice(namespace_bytes);
        
        Ok(CreateSessionParams {
            namespace_path,
            namespace_path_len: namespace_bytes.len() as u16,
            metadata: self.metadata,
            parent_session: self.parent_session,
        })
    }

    /// Create instruction for guard account creation
    pub fn create_guard_instruction(&self, guard_pubkey: Pubkey, session_pubkey: Pubkey) -> Result<Instruction> {
        let payer = self.client.payer();

        let accounts = vec![
            AccountMeta::new(guard_pubkey, false),
            AccountMeta::new(payer, true),
            AccountMeta::new_readonly(solana_sdk::system_program::ID, false),
        ];

        // Create instruction data
        let mut data = vec![];
        // Add discriminator for create_guard_account
        data.extend_from_slice(&anchor_lang::solana_program::hash::hash(b"global:create_guard_account").to_bytes()[..8]);
        data.extend_from_slice(&session_pubkey.to_bytes());
        data.push(self.allow_unregistered_cpi as u8);

        Ok(Instruction {
            program_id: valence_kernel::ID,
            accounts,
            data,
        })
    }

    /// Create instruction for session creation
    pub fn create_session_instruction(
        &self, 
        session_pubkey: Pubkey,
        alt_pubkey: Pubkey,
        guard_pubkey: Pubkey,
        shard: Pubkey,
    ) -> Result<Instruction> {
        let payer = self.client.payer();
        let params = self.build_params()?;

        let accounts = vec![
            AccountMeta::new(session_pubkey, false),
            AccountMeta::new(alt_pubkey, false),
            AccountMeta::new_readonly(guard_pubkey, false),
            AccountMeta::new(payer, true),
            AccountMeta::new_readonly(solana_sdk::system_program::ID, false),
        ];

        // Create instruction data
        let mut data = vec![];
        // Add discriminator for create_session_account
        data.extend_from_slice(&anchor_lang::solana_program::hash::hash(b"global:create_session_account").to_bytes()[..8]);
        data.extend_from_slice(&shard.to_bytes());
        data.extend_from_slice(&params.try_to_vec().unwrap());
        
        // Serialize initial registrations
        data.extend_from_slice(&(self.initial_borrowable.len() as u32).to_le_bytes());
        for account in &self.initial_borrowable {
            data.extend_from_slice(&account.try_to_vec().unwrap());
        }
        
        data.extend_from_slice(&(self.initial_programs.len() as u32).to_le_bytes());
        for program in &self.initial_programs {
            data.extend_from_slice(&program.try_to_vec().unwrap());
        }

        Ok(Instruction {
            program_id: valence_kernel::ID,
            accounts,
            data,
        })
    }
}

/// Handle to an active session
pub struct SessionHandle<'a> {
    client: &'a ValenceClient,
    session_pubkey: Pubkey,
    alt_pubkey: Pubkey,
}

impl<'a> SessionHandle<'a> {
    pub fn new(client: &'a ValenceClient, session_pubkey: Pubkey, alt_pubkey: Pubkey) -> Self {
        Self {
            client,
            session_pubkey,
            alt_pubkey,
        }
    }

    /// Create instruction to execute a batch of operations
    pub fn execute_batch_instruction(
        &self,
        batch: OperationBatch,
        guard_pubkey: Pubkey,
        cpi_allowlist: Pubkey,
        tx_submitter: Pubkey,
        remaining_accounts: Vec<AccountMeta>,
    ) -> Result<Instruction> {
        let caller = self.client.payer();

        let mut accounts = vec![
            AccountMeta::new(self.session_pubkey, false),
            AccountMeta::new_readonly(guard_pubkey, false),
            AccountMeta::new_readonly(self.alt_pubkey, false),
            AccountMeta::new_readonly(cpi_allowlist, false),
            AccountMeta::new_readonly(caller, true),
            AccountMeta::new_readonly(tx_submitter, true),
            AccountMeta::new_readonly(solana_sdk::sysvar::clock::ID, false),
        ];
        
        // Add remaining accounts for the operations
        accounts.extend(remaining_accounts);

        // Create instruction data
        let mut data = vec![];
        // Add discriminator for execute_batch
        data.extend_from_slice(&anchor_lang::solana_program::hash::hash(b"global:execute_batch").to_bytes()[..8]);
        data.extend_from_slice(&batch.try_to_vec().unwrap());

        Ok(Instruction {
            program_id: valence_kernel::ID,
            accounts,
            data,
        })
    }

    /// Create instruction to invalidate this session (for move semantics)
    pub fn invalidate_instruction(&self) -> Result<Instruction> {
        let owner = self.client.payer();

        let accounts = vec![
            AccountMeta::new(self.session_pubkey, false),
            AccountMeta::new_readonly(owner, true),
        ];

        // Create instruction data
        let mut data = vec![];
        // Add discriminator for invalidate_session
        data.extend_from_slice(&anchor_lang::solana_program::hash::hash(b"global:invalidate_session").to_bytes()[..8]);

        Ok(Instruction {
            program_id: valence_kernel::ID,
            accounts,
            data,
        })
    }

    /// Create instruction to manage ALT (add/remove accounts)
    pub fn manage_alt_instruction(
        &self,
        add_borrowable: Vec<RegisteredAccount>,
        add_programs: Vec<RegisteredProgram>,
        remove_accounts: Vec<Pubkey>,
    ) -> Result<Instruction> {
        let authority = self.client.payer();

        let accounts = vec![
            AccountMeta::new(self.alt_pubkey, false),
            AccountMeta::new_readonly(self.session_pubkey, false),
            AccountMeta::new_readonly(authority, true),
        ];

        // Create instruction data
        let mut data = vec![];
        // Add discriminator for manage_alt
        data.extend_from_slice(&anchor_lang::solana_program::hash::hash(b"global:manage_alt").to_bytes()[..8]);
        
        // Serialize parameters
        data.extend_from_slice(&(add_borrowable.len() as u32).to_le_bytes());
        for account in &add_borrowable {
            data.extend_from_slice(&account.try_to_vec().unwrap());
        }
        
        data.extend_from_slice(&(add_programs.len() as u32).to_le_bytes());
        for program in &add_programs {
            data.extend_from_slice(&program.try_to_vec().unwrap());
        }
        
        data.extend_from_slice(&(remove_accounts.len() as u32).to_le_bytes());
        for pubkey in &remove_accounts {
            data.extend_from_slice(&pubkey.to_bytes());
        }

        Ok(Instruction {
            program_id: valence_kernel::ID,
            accounts,
            data,
        })
    }
}

/// Helper to build operation batches
pub struct BatchBuilder {
    accounts: Vec<Pubkey>,
    operations: Vec<KernelOperation>,
}

impl BatchBuilder {
    pub fn new() -> Self {
        Self {
            accounts: Vec::new(),
            operations: Vec::new(),
        }
    }

    /// Add an account to the batch and return its index
    pub fn add_account(&mut self, account: Pubkey) -> u8 {
        if let Some(index) = self.accounts.iter().position(|&a| a == account) {
            index as u8
        } else {
            let index = self.accounts.len() as u8;
            self.accounts.push(account);
            index
        }
    }

    /// Add a borrow account operation
    pub fn borrow_account(&mut self, account: Pubkey, mode: u8) -> &mut Self {
        let index = self.add_account(account);
        self.operations.push(KernelOperation::BorrowAccount {
            account_index: index,
            mode,
        });
        self
    }

    /// Add a release account operation
    pub fn release_account(&mut self, account: Pubkey) -> &mut Self {
        let index = self.add_account(account);
        self.operations.push(KernelOperation::ReleaseAccount {
            account_index: index,
        });
        self
    }

    /// Add a call to registered function
    pub fn call_registered_function(
        &mut self,
        registry_id: u64,
        accounts: &[Pubkey],
        data: &[u8],
    ) -> Result<&mut Self> {
        if accounts.len() > MAX_CPI_ACCOUNT_INDICES {
            return Err(SdkError::InvalidOperation("Too many accounts".to_string()));
        }
        if data.len() > MAX_OPERATION_DATA_SIZE {
            return Err(SdkError::InvalidOperation("Data too large".to_string()));
        }

        let mut account_indices = [0u8; MAX_CPI_ACCOUNT_INDICES];
        for (i, account) in accounts.iter().enumerate() {
            account_indices[i] = self.add_account(*account);
        }

        let mut fixed_data = [0u8; MAX_OPERATION_DATA_SIZE];
        fixed_data[..data.len()].copy_from_slice(data);

        self.operations.push(KernelOperation::CallRegisteredFunction {
            registry_id,
            account_indices,
            account_indices_len: accounts.len() as u8,
            data: fixed_data,
            data_len: data.len() as u16,
        });
        
        Ok(self)
    }

    /// Build the operation batch
    pub fn build(self) -> Result<OperationBatch> {
        if self.accounts.len() > MAX_BATCH_ACCOUNTS {
            return Err(SdkError::InvalidOperation("Too many accounts in batch".to_string()));
        }
        if self.operations.len() > MAX_BATCH_OPERATIONS {
            return Err(SdkError::InvalidOperation("Too many operations in batch".to_string()));
        }

        let mut accounts = [Pubkey::default(); MAX_BATCH_ACCOUNTS];
        accounts[..self.accounts.len()].copy_from_slice(&self.accounts);

        let operations_len = self.operations.len() as u8;
        let mut operations: [Option<KernelOperation>; MAX_BATCH_OPERATIONS] = [const { None }; MAX_BATCH_OPERATIONS];
        for (i, op) in self.operations.into_iter().enumerate() {
            operations[i] = Some(op);
        }

        Ok(OperationBatch {
            accounts,
            accounts_len: self.accounts.len() as u8,
            operations,
            operations_len,
        })
    }
}