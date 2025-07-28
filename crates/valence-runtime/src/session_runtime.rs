// Session runtime - Off-chain session management and account abstraction
use crate::{Result, RuntimeError};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{pubkey::Pubkey, signature::Signature};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use valence_kernel::{
    operations::{OperationBatch, SessionOperation},
    state::{Session, BorrowedAccount},
};

/// Off-chain session manager
pub struct SessionManager {
    rpc_client: Arc<RpcClient>,
    sessions: Arc<RwLock<HashMap<Pubkey, SessionState>>>,
}

/// Off-chain session state tracking
pub struct SessionState {
    pub session_pubkey: Pubkey,
    pub on_chain_state: Session,
    pub borrowed_accounts: HashMap<Pubkey, AccountState>,
    pub pending_operations: Vec<SessionOperation>,
    pub last_sync_slot: u64,
}

/// Off-chain account state
pub struct AccountState {
    pub address: Pubkey,
    pub data: Vec<u8>,
    pub owner: Pubkey,
    pub lamports: u64,
    pub borrow_mode: u8,
    pub last_modified_slot: u64,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new(rpc_client: Arc<RpcClient>) -> Self {
        Self {
            rpc_client,
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Load a session from chain
    pub async fn load_session(&self, session_pubkey: Pubkey) -> Result<()> {
        let account = self.rpc_client
            .get_account(&session_pubkey)
            .await
            .map_err(|e| RuntimeError::RpcError(e.to_string()))?;
            
        let session = Session::try_from_slice(&account.data[8..])
            .map_err(|_| RuntimeError::InvalidAccountData)?;
            
        let state = SessionState {
            session_pubkey,
            on_chain_state: session,
            borrowed_accounts: HashMap::new(),
            pending_operations: Vec::new(),
            last_sync_slot: 0,
        };
        
        let mut sessions = self.sessions.write().await;
        sessions.insert(session_pubkey, state);
        
        Ok(())
    }
    
    /// Sync borrowed accounts for a session
    pub async fn sync_borrowed_accounts(&self, session_pubkey: Pubkey) -> Result<()> {
        let sessions = self.sessions.read().await;
        let session_state = sessions.get(&session_pubkey)
            .ok_or_else(|| RuntimeError::SessionNotFound(session_pubkey.to_string()))?;
            
        // Get accounts that are borrowed on-chain
        let borrowed_accounts = &session_state.on_chain_state.borrowed_accounts;
        let borrowed_count = session_state.on_chain_state.borrowed_count;
        
        drop(sessions); // Release read lock
        
        // Fetch each borrowed account
        for i in 0..borrowed_count as usize {
            let borrowed = &borrowed_accounts[i];
            if !borrowed.is_empty() {
                self.sync_account(&session_pubkey, &borrowed.address, borrowed.mode).await?;
            }
        }
        
        Ok(())
    }
    
    /// Sync a specific account
    async fn sync_account(
        &self, 
        session_pubkey: &Pubkey, 
        account: &Pubkey,
        mode: u8
    ) -> Result<()> {
        let account_info = self.rpc_client
            .get_account(account)
            .await
            .map_err(|e| RuntimeError::RpcError(e.to_string()))?;
            
        let account_state = AccountState {
            address: *account,
            data: account_info.data.clone(),
            owner: account_info.owner,
            lamports: account_info.lamports,
            borrow_mode: mode,
            last_modified_slot: 0, // Would get from account data
        };
        
        let mut sessions = self.sessions.write().await;
        if let Some(session_state) = sessions.get_mut(session_pubkey) {
            session_state.borrowed_accounts.insert(*account, account_state);
        }
        
        Ok(())
    }
    
    /// Queue an operation for execution
    pub async fn queue_operation(
        &self,
        session_pubkey: Pubkey,
        operation: SessionOperation
    ) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        let session_state = sessions.get_mut(&session_pubkey)
            .ok_or_else(|| RuntimeError::SessionNotFound(session_pubkey.to_string()))?;
            
        session_state.pending_operations.push(operation);
        Ok(())
    }
    
    /// Build operation batch from queued operations
    pub async fn build_batch(
        &self,
        session_pubkey: Pubkey,
        auto_release: bool
    ) -> Result<OperationBatch> {
        let mut sessions = self.sessions.write().await;
        let session_state = sessions.get_mut(&session_pubkey)
            .ok_or_else(|| RuntimeError::SessionNotFound(session_pubkey.to_string()))?;
            
        let operations = std::mem::take(&mut session_state.pending_operations);
        
        Ok(OperationBatch {
            operations,
            auto_release,
        })
    }
    
    /// Simulate operations locally
    pub async fn simulate_operations(
        &self,
        session_pubkey: Pubkey,
        operations: &[SessionOperation]
    ) -> Result<SimulationResult> {
        let sessions = self.sessions.read().await;
        let session_state = sessions.get(&session_pubkey)
            .ok_or_else(|| RuntimeError::SessionNotFound(session_pubkey.to_string()))?;
            
        let mut borrowed_accounts = session_state.borrowed_accounts.clone();
        let mut compute_units = 0u64;
        let mut effects = Vec::new();
        
        for operation in operations {
            match operation {
                SessionOperation::BorrowAccount { account, mode } => {
                    if borrowed_accounts.contains_key(account) {
                        return Err(RuntimeError::OperationFailed(
                            "Account already borrowed".to_string()
                        ));
                    }
                    
                    // Simulate borrowing
                    let account_state = AccountState {
                        address: *account,
                        data: vec![], // Would fetch in real implementation
                        owner: Pubkey::default(),
                        lamports: 0,
                        borrow_mode: *mode,
                        last_modified_slot: 0,
                    };
                    
                    borrowed_accounts.insert(*account, account_state);
                    effects.push(format!("Borrow account {}", account));
                    compute_units += 5_000;
                }
                
                SessionOperation::ReleaseAccount { account } => {
                    if !borrowed_accounts.contains_key(account) {
                        return Err(RuntimeError::OperationFailed(
                            "Account not borrowed".to_string()
                        ));
                    }
                    
                    borrowed_accounts.remove(account);
                    effects.push(format!("Release account {}", account));
                    compute_units += 3_000;
                }
                
                SessionOperation::InvokeProgram { program, data, account_indices } => {
                    // Verify all accounts are available
                    let borrowed: Vec<_> = borrowed_accounts.keys().cloned().collect();
                    for &index in account_indices {
                        if index as usize >= borrowed.len() {
                            return Err(RuntimeError::OperationFailed(
                                "Invalid account index".to_string()
                            ));
                        }
                    }
                    
                    effects.push(format!("Invoke program {} with {} bytes", program, data.len()));
                    compute_units += 50_000;
                }
                
                SessionOperation::UpdateMetadata { metadata } => {
                    effects.push(format!("Update metadata"));
                    compute_units += 2_000;
                }
                
                SessionOperation::Custom { discriminator, data } => {
                    effects.push(format!("Custom operation {:?}", discriminator));
                    compute_units += 10_000;
                }
            }
        }
        
        Ok(SimulationResult {
            success: true,
            compute_units,
            effects,
            final_borrowed_count: borrowed_accounts.len(),
        })
    }
    
    /// Get session state
    pub async fn get_session(&self, session_pubkey: &Pubkey) -> Result<SessionState> {
        let sessions = self.sessions.read().await;
        sessions.get(session_pubkey)
            .cloned()
            .ok_or_else(|| RuntimeError::SessionNotFound(session_pubkey.to_string()))
    }
}

/// Result of operation simulation
#[derive(Debug)]
pub struct SimulationResult {
    pub success: bool,
    pub compute_units: u64,
    pub effects: Vec<String>,
    pub final_borrowed_count: usize,
}

// Clone implementations for state types
impl Clone for SessionState {
    fn clone(&self) -> Self {
        Self {
            session_pubkey: self.session_pubkey,
            on_chain_state: self.on_chain_state.clone(),
            borrowed_accounts: self.borrowed_accounts.clone(),
            pending_operations: self.pending_operations.clone(),
            last_sync_slot: self.last_sync_slot,
        }
    }
}

impl Clone for AccountState {
    fn clone(&self) -> Self {
        Self {
            address: self.address,
            data: self.data.clone(),
            owner: self.owner,
            lamports: self.lamports,
            borrow_mode: self.borrow_mode,
            last_modified_slot: self.last_modified_slot,
        }
    }
}

// Helper to convert on-chain session to off-chain representation
impl SessionState {
    /// Update from on-chain state
    pub fn update_from_chain(&mut self, session: Session) {
        self.on_chain_state = session;
    }
    
    /// Check if a specific account is borrowed
    pub fn is_account_borrowed(&self, account: &Pubkey) -> bool {
        self.borrowed_accounts.contains_key(account)
    }
    
    /// Get borrowed account state
    pub fn get_borrowed_account(&self, account: &Pubkey) -> Option<&AccountState> {
        self.borrowed_accounts.get(account)
    }
}