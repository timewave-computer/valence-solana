use anchor_lang::prelude::*;
use anchor_lang::{InstructionData, ToAccountMetas};
use solana_program_test::*;
use solana_sdk::{
    account::Account,
    commitment_config::CommitmentConfig,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    rent::Rent,
    signature::{Keypair, Signer},
    system_instruction,
    system_program,
    sysvar::{self, clock::Clock},
    transaction::Transaction,
};

// Direct import to avoid macro issues
use valence_kernel::{
    self, PROGRAM_ID, CpiAllowlistAccount, SerializedGuard, GuardData,
    CreateSessionParams, Session, KernelOperationBatch as OperationBatch,
    KernelError as ValenceError
};

pub struct TestContext {
    pub banks_client: BanksClient,
    pub payer: Keypair,
    pub authority: Keypair,
    pub recent_blockhash: solana_sdk::hash::Hash,
    pub program_id: Pubkey,
}

impl TestContext {
    pub async fn new() -> Self {
        let program_id = valence_kernel::PROGRAM_ID;
        let mut program_test = ProgramTest::new(
            "valence_kernel",
            program_id,
            processor!(valence_kernel::entry),
        );
        
        let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
        let authority = Keypair::new();
        
        // Fund authority
        let transfer_ix = system_instruction::transfer(
            &payer.pubkey(),
            &authority.pubkey(),
            10_000_000_000, // 10 SOL
        );
        
        let mut tx = Transaction::new_with_payer(
            &[transfer_ix],
            Some(&payer.pubkey()),
        );
        tx.sign(&[&payer], recent_blockhash);
        banks_client.process_transaction(tx).await.unwrap();
        
        Self {
            banks_client,
            payer: authority.insecure_clone(),
            authority,
            recent_blockhash,
            program_id,
        }
    }
    
    pub async fn initialize_program(&mut self) -> Result<()> {
        let ix = Instruction {
            program_id: self.program_id,
            accounts: valence_kernel::accounts::Initialize {
                authority: self.authority.pubkey(),
                system_program: system_program::ID,
            }
            .to_account_metas(None),
            data: valence_kernel::instruction::Initialize {}.data(),
        };
        
        self.send_transaction(&[ix], &[&self.payer]).await
    }
    
    pub async fn initialize_allowlist(&mut self, allowlist: &Keypair) -> Result<()> {
        let rent = self.banks_client.get_rent().await.unwrap();
        let space = 8 + std::mem::size_of::<CpiAllowlistAccount>();
        let lamports = rent.minimum_balance(space);
        
        let create_account_ix = system_instruction::create_account(
            &self.payer.pubkey(),
            &allowlist.pubkey(),
            lamports,
            space as u64,
            &self.program_id,
        );
        
        let initialize_ix = Instruction {
            program_id: self.program_id,
            accounts: valence_kernel::accounts::InitializeAllowlist {
                cpi_allowlist: allowlist.pubkey(),
                authority: self.payer.pubkey(),
                system_program: system_program::ID,
            }
            .to_account_metas(None),
            data: valence_kernel::instruction::InitializeAllowlist {}.data(),
        };
        
        self.send_transaction(
            &[create_account_ix, initialize_ix],
            &[&self.payer, allowlist],
        ).await
    }
    
    pub async fn add_to_allowlist(
        &mut self,
        allowlist: &Keypair,
        program_id: Pubkey,
    ) -> Result<()> {
        let ix = Instruction {
            program_id: self.program_id,
            accounts: valence_kernel::accounts::ManageAllowlist {
                cpi_allowlist: allowlist.pubkey(),
                authority: self.payer.pubkey(),
            }
            .to_account_metas(None),
            data: valence_kernel::instruction::AddToAllowlist { program_id }.data(),
        };
        
        self.send_transaction(&[ix], &[&self.payer]).await
    }
    
    pub async fn remove_from_allowlist(
        &mut self,
        allowlist: &Keypair,
        program_id: Pubkey,
    ) -> Result<()> {
        let ix = Instruction {
            program_id: self.program_id,
            accounts: valence_kernel::accounts::ManageAllowlist {
                cpi_allowlist: allowlist.pubkey(),
                authority: self.payer.pubkey(),
            }
            .to_account_metas(None),
            data: valence_kernel::instruction::RemoveFromAllowlist { program_id }.data(),
        };
        
        self.send_transaction(&[ix], &[&self.payer]).await
    }
    
    pub async fn create_guard_data(
        &mut self,
        guard_data: &Keypair,
        session: Pubkey,
        serialized_guard: SerializedGuard,
    ) -> Result<()> {
        let rent = self.banks_client.get_rent().await.unwrap();
        let space = GuardData::calculate_space_for_apu_program(&serialized_guard);
        let lamports = rent.minimum_balance(space);
        
        let create_account_ix = system_instruction::create_account(
            &self.payer.pubkey(),
            &guard_data.pubkey(),
            lamports,
            space as u64,
            &self.program_id,
        );
        
        let create_guard_ix = Instruction {
            program_id: self.program_id,
            accounts: valence_kernel::accounts::CreateGuardData {
                guard_data: guard_data.pubkey(),
                payer: self.payer.pubkey(),
                system_program: system_program::ID,
            }
            .to_account_metas(None),
            data: valence_kernel::instruction::CreateGuardData {
                session,
                serialized_guard,
            }
            .data(),
        };
        
        self.send_transaction(
            &[create_account_ix, create_guard_ix],
            &[&self.payer, guard_data],
        ).await
    }
    
    pub async fn create_session_account(
        &mut self,
        session: &Keypair,
        shard: Pubkey,
        params: CreateSessionParams,
    ) -> Result<()> {
        let rent = self.banks_client.get_rent().await.unwrap();
        let space = 8 + std::mem::size_of::<Session>();
        let lamports = rent.minimum_balance(space);
        
        let create_account_ix = system_instruction::create_account(
            &self.payer.pubkey(),
            &session.pubkey(),
            lamports,
            space as u64,
            &self.program_id,
        );
        
        let create_session_account_ix = Instruction {
            program_id: self.program_id,
            accounts: valence_kernel::accounts::CreateSession {
                session: session.pubkey(),
                owner: self.payer.pubkey(),
                system_program: system_program::ID,
            }
            .to_account_metas(None),
            data: valence_kernel::instruction::CreateSession {
                shard,
                params,
            }
            .data(),
        };
        
        self.send_transaction(
            &[create_account_ix, create_session_account_ix],
            &[&self.payer, session],
        ).await
    }
    
    pub async fn execute_session_operations(
        &mut self,
        session: &Keypair,
        guard_data: &Keypair,
        allowlist: &Keypair,
        batch: OperationBatch,
        remaining_accounts: Vec<&Keypair>,
    ) -> Result<()> {
        let mut account_metas = valence_kernel::accounts::ExecuteOperations {
            session: session.pubkey(),
            guard_data: guard_data.pubkey(),
            cpi_allowlist: allowlist.pubkey(),
            binding_session: None,
            binding_guard_data: None,
            caller: self.payer.pubkey(),
            clock: sysvar::clock::id(),
        }
        .to_account_metas(None);
        
        // Add remaining accounts
        for account in &remaining_accounts {
            account_metas.push(AccountMeta::new(account.pubkey(), false));
        }
        
        let ix = Instruction {
            program_id: self.program_id,
            accounts: account_metas,
            data: valence_kernel::instruction::ExecuteOperations { batch }.data(),
        };
        
        self.send_transaction(&[ix], &[&self.payer]).await
    }
    
    pub async fn get_account(&mut self, pubkey: &Pubkey) -> Account {
        self.banks_client
            .get_account(*pubkey)
            .await
            .unwrap()
            .expect("Account not found")
    }
    
    pub async fn get_clock(&mut self) -> Clock {
        self.banks_client.get_sysvar::<Clock>().await.unwrap()
    }
    
    pub async fn warp_to_timestamp(&mut self, timestamp: i64) {
        // Note: solana-program-test doesn't support clock manipulation directly
        // In real tests, you would use the warp_to_slot functionality
        // For now, this is a placeholder
        println!("Clock warping to timestamp {} (not implemented in test framework)", timestamp);
    }
    
    async fn send_transaction(
        &mut self,
        instructions: &[Instruction],
        signers: &[&Keypair],
    ) -> Result<()> {
        let recent_blockhash = self.banks_client.get_latest_blockhash().await.unwrap();
        
        let mut transaction = Transaction::new_with_payer(
            instructions,
            Some(&self.payer.pubkey()),
        );
        
        transaction.sign(signers, recent_blockhash);
        
        self.banks_client
            .process_transaction(transaction)
            .await
            .map_err(|e| error!(ValenceError::InvalidParameters))?;
        
        Ok(())
    }
}

// Export commonly used constants
pub use valence_kernel::{ACCESS_MODE_READ, ACCESS_MODE_WRITE, ACCESS_MODE_READ_WRITE};
pub use valence_kernel::validation::{MAX_CPI_DATA_SIZE, MAX_CUSTOM_DATA_SIZE};