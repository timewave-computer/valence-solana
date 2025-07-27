use crate::{Result, ValenceClient};
use anchor_lang::prelude::*;
use solana_sdk::instruction::Instruction;
use valence_core::{
    guards::Guard,
    state::{CreateSessionParams, Session, SessionScope, SessionSharedData},
    operations::OperationBatch,
};

/// Builder for creating sessions
pub struct SessionBuilder<'a> {
    client: &'a ValenceClient,
    scope: SessionScope,
    guard: Option<Guard>,
    bound_to: Option<Pubkey>,
    shared_data: SessionSharedData,
    metadata: [u8; 64],
}

impl<'a> SessionBuilder<'a> {
    /// Create a new session builder
    pub fn new(client: &'a ValenceClient) -> Self {
        Self {
            client,
            scope: SessionScope::User,
            guard: None,
            bound_to: None,
            shared_data: SessionSharedData::default(),
            metadata: [0u8; 64],
        }
    }

    /// Set the session scope
    pub fn scope(mut self, scope: SessionScope) -> Self {
        self.scope = scope;
        self
    }

    /// Set the guard
    pub fn guard(mut self, guard: Guard) -> Self {
        self.guard = Some(guard);
        self
    }

    /// Set the binding to another session
    pub fn bound_to(mut self, binding: Pubkey) -> Self {
        self.bound_to = Some(binding);
        self
    }

    /// Set metadata
    pub fn metadata(mut self, metadata: [u8; 64]) -> Self {
        self.metadata = metadata;
        self
    }

    /// Build the CreateSessionParams
    pub fn build_params(&self, guard_data: Pubkey) -> Result<CreateSessionParams> {
        Ok(CreateSessionParams {
            scope: self.scope,
            guard_data,
            bound_to: self.bound_to,
            shared_data: self.shared_data,
            metadata: self.metadata,
        })
    }

    /// Create instruction for session creation
    pub fn create_instruction(self, session_pubkey: Pubkey, guard_data: Pubkey, protocol: Pubkey) -> Result<Instruction> {
        let payer = self.client.payer();
        let params = self.build_params(guard_data)?;

        let account_metas = vec![
            AccountMeta::new(session_pubkey, false),
            AccountMeta::new(payer, true),
            AccountMeta::new_readonly(solana_sdk::system_program::ID, false),
        ];

        // Manually create instruction discriminator and data
        // This is a simplified approach - in practice you'd use anchor's codegen
        let mut instruction_data = Vec::new();
        instruction_data.extend_from_slice(&[0u8; 8]); // Placeholder discriminator
        instruction_data.extend_from_slice(&protocol.to_bytes());
        instruction_data.extend_from_slice(&params.try_to_vec().unwrap());

        Ok(Instruction {
            program_id: valence_core::ID,
            accounts: account_metas,
            data: instruction_data,
        })
    }
}

/// Extension methods for operations
impl<'a> SessionHandle<'a> {
    /// Execute a batch of operations
    pub fn execute_operations_instruction(
        &self,
        batch: OperationBatch,
        additional_accounts: &[Pubkey]
    ) -> Instruction {
        let mut account_metas = vec![
            AccountMeta::new(self.session, false),
            AccountMeta::new_readonly(self.client.payer(), true),
            AccountMeta::new_readonly(solana_sdk::sysvar::clock::ID, false),
        ];
        
        // Add binding session if present
        if let Some(binding) = self.bound_to {
            account_metas.push(AccountMeta::new(binding, false));
        }
        
        // Add all additional accounts
        for account in additional_accounts {
            account_metas.push(AccountMeta::new(*account, false));
        }
        
        // Manually create instruction data
        let mut instruction_data = Vec::new();
        instruction_data.extend_from_slice(&[1u8; 8]); // Placeholder discriminator for execute_operations
        instruction_data.extend_from_slice(&batch.try_to_vec().unwrap());
        
        Instruction {
            program_id: valence_core::ID,
            accounts: account_metas,
            data: instruction_data,
        }
    }
}

/// Session handle for executing operations
pub struct SessionHandle<'a> {
    client: &'a ValenceClient,
    session: Pubkey,
    bound_to: Option<Pubkey>,
}

impl<'a> SessionHandle<'a> {
    /// Create a new session handle
    pub fn new(client: &'a ValenceClient, session: Pubkey) -> Self {
        Self {
            client,
            session,
            bound_to: None,
        }
    }

    /// Set binding session for hierarchical operations
    pub fn with_binding(mut self, binding: Pubkey) -> Self {
        self.bound_to = Some(binding);
        self
    }

    /// This method is deprecated - use execute_operations_instruction instead
    #[deprecated(note = "Use execute_operations_instruction with OperationBatch")]
    pub fn execute_instruction(&self, _operation: Vec<u8>) -> Instruction {
        unimplemented!("Use execute_operations_instruction with OperationBatch instead")
    }

    /// Get the session account
    pub fn get_session(&self) -> Result<Session> {
        self.client.get_account(&self.session)
    }
}