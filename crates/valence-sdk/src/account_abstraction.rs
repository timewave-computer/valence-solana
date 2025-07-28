use crate::{Result, SdkError};
use anchor_lang::prelude::*;
use solana_sdk::instruction::AccountMeta;
use std::collections::HashMap;
use valence_kernel::{OperationBatch, SessionOperation, ProgramManifestEntry};

/// Manages account abstraction for sessions
pub struct AccountManager {
    /// Accounts tracked by this manager
    accounts: HashMap<Pubkey, ManagedAccount>,
}

/// A managed account with type information
pub struct ManagedAccount {
    pub address: Pubkey,
    pub account_type: AccountType,
    pub data: Vec<u8>,
    pub is_borrowed: bool,
    pub borrow_mode: Option<u8>,
}

/// Type information for managed accounts
#[derive(Clone, Debug)]
pub enum AccountType {
    Token { mint: Pubkey },
    TokenAccount { mint: Pubkey, owner: Pubkey },
    Data { discriminator: [u8; 8] },
    Native,
}

impl Default for AccountManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AccountManager {
    /// Create a new account manager
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
        }
    }
    
    /// Register an account for management
    pub fn register_account(
        &mut self, 
        address: Pubkey, 
        account_type: AccountType,
        data: Vec<u8>
    ) -> Result<()> {
        if self.accounts.contains_key(&address) {
            return Err(SdkError::AccountAlreadyRegistered);
        }
        
        self.accounts.insert(address, ManagedAccount {
            address,
            account_type,
            data,
            is_borrowed: false,
            borrow_mode: None,
        });
        
        Ok(())
    }
    
    /// Get a managed account
    pub fn get_account(&self, address: &Pubkey) -> Option<&ManagedAccount> {
        self.accounts.get(address)
    }
    
    /// Mark account as borrowed
    pub fn mark_borrowed(&mut self, address: &Pubkey, mode: u8) -> Result<()> {
        let account = self.accounts.get_mut(address)
            .ok_or(SdkError::AccountNotFound(address.to_string()))?;
            
        if account.is_borrowed {
            return Err(SdkError::AccountAlreadyBorrowed);
        }
        
        account.is_borrowed = true;
        account.borrow_mode = Some(mode);
        Ok(())
    }
    
    /// Mark account as released
    pub fn mark_released(&mut self, address: &Pubkey) -> Result<()> {
        let account = self.accounts.get_mut(address)
            .ok_or(SdkError::AccountNotFound(address.to_string()))?;
            
        account.is_borrowed = false;
        account.borrow_mode = None;
        Ok(())
    }
    
    /// Build account metas for borrowed accounts
    pub fn build_account_metas(&self, indices: &[u8]) -> Result<Vec<AccountMeta>> {
        let mut borrowed: Vec<_> = self.accounts.values()
            .filter(|a| a.is_borrowed)
            .collect();
        borrowed.sort_by_key(|a| a.address);
        
        let mut metas = Vec::new();
        for &index in indices {
            let account = borrowed.get(index as usize)
                .ok_or(SdkError::InvalidAccountIndex)?;
                
            let is_writable = account.borrow_mode
                .map(|m| m & 2 != 0)
                .unwrap_or(false);
                
            metas.push(AccountMeta {
                pubkey: account.address,
                is_signer: false,
                is_writable,
            });
        }
        
        Ok(metas)
    }
}

/// Builder for creating operation batches with account management
pub struct OperationBuilder {
    operations: Vec<SessionOperation>,
    account_manager: AccountManager,
    auto_release: bool,
    program_manifest: Vec<ProgramManifestEntry>,
}

impl Default for OperationBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl OperationBuilder {
    /// Create a new operation builder
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
            account_manager: AccountManager::new(),
            auto_release: true,
            program_manifest: Vec::new(),
        }
    }
    
    /// Register an account for use in operations
    pub fn register_account(
        mut self,
        address: Pubkey,
        account_type: AccountType,
        data: Vec<u8>
    ) -> Result<Self> {
        self.account_manager.register_account(address, account_type, data)?;
        Ok(self)
    }
    
    /// Borrow an account
    pub fn borrow_account(mut self, address: Pubkey, mode: u8) -> Result<Self> {
        // Verify account is registered
        if !self.account_manager.accounts.contains_key(&address) {
            return Err(SdkError::AccountNotFound(address.to_string()));
        }
        
        self.operations.push(SessionOperation::BorrowAccount {
            account: address,
            mode,
        });
        
        self.account_manager.mark_borrowed(&address, mode)?;
        Ok(self)
    }
    
    /// Release an account
    pub fn release_account(mut self, address: Pubkey) -> Result<Self> {
        self.operations.push(SessionOperation::ReleaseAccount {
            account: address,
        });
        
        self.account_manager.mark_released(&address)?;
        Ok(self)
    }
    
    /// Add a CPI operation
    pub fn invoke_program(
        mut self,
        program: Pubkey,
        data: Vec<u8>,
        accounts: &[Pubkey]
    ) -> Result<Self> {
        // Check if program is already in manifest
        let manifest_index = if let Some(index) = self.program_manifest.iter().position(|p| p.program_id == program) {
            index as u8
        } else {
            // Add to manifest
            let index = self.program_manifest.len() as u8;
            self.program_manifest.push(ProgramManifestEntry {
                program_id: program,
            });
            index
        };
        
        // Build account indices from borrowed accounts
        let borrowed: Vec<_> = self.account_manager.accounts.values()
            .filter(|a| a.is_borrowed)
            .map(|a| a.address)
            .collect();
            
        let mut indices = Vec::new();
        for account in accounts {
            let index = borrowed.iter()
                .position(|a| a == account)
                .ok_or(SdkError::AccountNotBorrowed)?;
            indices.push(index as u8);
        }
        
        self.operations.push(SessionOperation::CallProgram {
            manifest_index,
            data,
            account_indices: indices,
        });
        
        Ok(self)
    }
    
    /// Update session metadata
    pub fn update_metadata(mut self, metadata: [u8; 64]) -> Self {
        self.operations.push(SessionOperation::UpdateMetadata {
            metadata,
        });
        self
    }
    
    /// Set auto-release behavior
    pub fn auto_release(mut self, auto: bool) -> Self {
        self.auto_release = auto;
        self
    }
    
    /// Build the operation batch
    pub fn build(self) -> OperationBatch {
        OperationBatch {
            operations: self.operations,
            auto_release: self.auto_release,
            program_manifest: self.program_manifest,
        }
    }
    
    /// Get all registered accounts for inclusion in transaction
    pub fn get_all_accounts(&self) -> Vec<Pubkey> {
        self.account_manager.accounts.keys().copied().collect()
    }
}