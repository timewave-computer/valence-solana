//! Core Session Builder implementation

use anyhow::{Context, Result};
use backoff::{backoff::Backoff, ExponentialBackoff};
use futures::StreamExt;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::{Keypair, Signature, Signer},
    system_instruction,
    transaction::Transaction,
};
use std::{
    collections::BTreeMap,
    str::FromStr,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, SystemTime},
};
use tokio::{
    sync::Semaphore,
    task::JoinHandle,
    time::sleep,
};
use tracing::{debug, error, info, warn};
use valence_domain_clients::solana::{SolanaRpcClient, SolanaSigningClient, SolanaBaseClient};

use crate::{
    config::Config,
    error::SessionBuilderError,
    event_monitor::{EventMonitor, PDAComputedEvent},
    metrics::Metrics,
};

/// Core Session Builder service
pub struct SessionBuilder<C: SolanaRpcClient + SolanaSigningClient + SolanaBaseClient + Send + Sync + Clone> {
    config: Arc<Config>,
    client: Arc<C>,
    event_monitor: EventMonitor,
    metrics: Arc<Metrics>,
    is_running: Arc<AtomicBool>,
    accounts_created: Arc<AtomicU64>,
    accounts_failed: Arc<AtomicU64>,
    semaphore: Arc<Semaphore>,
}

impl<C: SolanaRpcClient + SolanaSigningClient + SolanaBaseClient + Send + Sync + Clone> SessionBuilder<C> {
    /// Create a new SessionBuilder instance
    pub async fn new(config: Arc<Config>, client: Arc<C>) -> Result<Self> {
        // Create RPC client for event monitor (still needs traditional client)
        let rpc_client = Arc::new(solana_client::rpc_client::RpcClient::new_with_commitment(
            config.rpc_url.clone(),
            CommitmentConfig::confirmed(),
        ));
        
        let event_monitor = EventMonitor::new(
            rpc_client,
            config.session_factory_program_id()?,
        ).await?;
        
        let metrics = Arc::new(Metrics::new());
        
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_creations));
        
        // Verify connection using valence domain client
        Self::verify_setup(&client).await?;
        
        Ok(SessionBuilder {
            config,
            client,
            event_monitor,
            metrics,
            is_running: Arc::new(AtomicBool::new(false)),
            accounts_created: Arc::new(AtomicU64::new(0)),
            accounts_failed: Arc::new(AtomicU64::new(0)),
            semaphore,
        })
    }
    
    /// Run the session builder service
    pub async fn run(&self) -> Result<()> {
        if self.is_running.load(Ordering::Relaxed) {
            return Err(SessionBuilderError::AlreadyRunning.into());
        }
        
        self.is_running.store(true, Ordering::Relaxed);
        info!("Starting Session Builder service");
        
        // Start event monitoring
        let mut event_stream = self.event_monitor.start().await?;
        
        // Start health check task
        let health_check_handle = self.start_health_check_task();
        
        // Process events
        while let Some(event) = event_stream.next().await {
            if !self.is_running.load(Ordering::Relaxed) {
                break;
            }
            
            match event {
                Ok(pda_event) => {
                    let builder = self.clone();
                    tokio::spawn(async move {
                        if let Err(e) = builder.handle_pda_computed_event(pda_event).await {
                            error!("Failed to handle PDA computed event: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Event monitoring error: {}", e);
                    // Continue processing other events
                }
            }
        }
        
        health_check_handle.abort();
        info!("Session Builder service stopped");
        Ok(())
    }
    
    /// Stop the session builder service
    pub fn stop(&self) {
        info!("Stopping Session Builder service");
        self.is_running.store(false, Ordering::Relaxed);
    }
    
    /// Handle a PDA computed event
    async fn handle_pda_computed_event(&self, event: PDAComputedEvent) -> Result<()> {
        let start_time = SystemTime::now();
        info!("Processing PDA computed event for: {}", event.expected_pda);
        
        // Acquire semaphore permit for concurrency control
        let _permit = self.semaphore.acquire().await?;
        
        // Record metric
        self.metrics.record_event_received();
        
        // Execute with retry logic
        let result = self.create_account_with_retry(&event).await;
        
        match result {
            Ok(signature) => {
                self.accounts_created.fetch_add(1, Ordering::Relaxed);
                self.metrics.record_account_created();
                info!(
                    "Successfully created account {} with signature {} (duration: {:?})",
                    event.expected_pda,
                    signature,
                    start_time.elapsed().unwrap_or_default()
                );
            }
            Err(e) => {
                self.accounts_failed.fetch_add(1, Ordering::Relaxed);
                self.metrics.record_account_failed();
                error!(
                    "Failed to create account {}: {} (duration: {:?})",
                    event.expected_pda,
                    e,
                    start_time.elapsed().unwrap_or_default()
                );
            }
        }
        
        Ok(())
    }
    
    /// Create account with exponential backoff retry
    async fn create_account_with_retry(&self, event: &PDAComputedEvent) -> Result<Signature> {
        let mut backoff = ExponentialBackoff::default();
        backoff.max_interval = Duration::from_secs(30);
        backoff.max_elapsed_time = Some(Duration::from_secs(300)); // 5 minutes
        
        loop {
            match self.create_account(event).await {
                Ok(signature) => return Ok(signature),
                Err(e) => {
                    // Check if error is retryable
                    if !self.is_retryable_error(&e) {
                        return Err(e);
                    }
                    
                    // Get next backoff duration
                    match backoff.next_backoff() {
                        Some(duration) => {
                            warn!(
                                "Retrying account creation for {} after {:?}: {}",
                                event.expected_pda, duration, e
                            );
                            sleep(duration).await;
                        }
                        None => {
                            error!("Max retry attempts reached for {}", event.expected_pda);
                            return Err(e);
                        }
                    }
                }
            }
        }
    }
    
    /// Check if an error is retryable
    fn is_retryable_error(&self, error: &anyhow::Error) -> bool {
        let error_str = error.to_string();
        
        // Network errors
        if error_str.contains("connection") || 
           error_str.contains("timeout") ||
           error_str.contains("transport") {
            return true;
        }
        
        // RPC errors that might be transient
        if error_str.contains("blockhash not found") ||
           error_str.contains("too many requests") ||
           error_str.contains("service unavailable") {
            return true;
        }
        
        // Account already exists is not retryable
        if error_str.contains("already exists") {
            return false;
        }
        
        // Default to not retryable
        false
    }
    
    /// Create a single account
    async fn create_account(&self, event: &PDAComputedEvent) -> Result<Signature> {
        // For now, we'll use a hybrid approach with the traditional client
        // TODO: Once valence-domain-clients has full account creation support, update this
        
        // Check if account already exists using valence-domain-client
        let account_exists = self.client.account_exists(&event.expected_pda.to_string()).await?;
        if account_exists {
            return Err(SessionBuilderError::AccountAlreadyExists(event.expected_pda).into());
        }
        
        // For account creation, we still need to use the traditional approach
        // This is because valence-domain-clients doesn't yet support creating arbitrary accounts
        let keypair = Keypair::from_bytes(&self.config.load_keypair()?.to_bytes())
            .context("Failed to load keypair")?;
        
        // Create temporary RPC client for account creation
        let rpc_client = solana_client::rpc_client::RpcClient::new_with_commitment(
            self.config.rpc_url.clone(),
            CommitmentConfig::confirmed(),
        );
        
        // Calculate rent exemption
        let rent_exemption = rpc_client
            .get_minimum_balance_for_rent_exemption(event.expected_size)
            .context("Failed to get rent exemption amount")?;
        
        // Create account instruction
        let create_account_ix = system_instruction::create_account(
            &keypair.pubkey(),
            &event.expected_pda,
            rent_exemption,
            event.expected_size as u64,
            &event.expected_owner,
        );
        
        // Create and send transaction
        let recent_blockhash = rpc_client
            .get_latest_blockhash()
            .context("Failed to get recent blockhash")?;
        
        let transaction = Transaction::new_signed_with_payer(
            &[create_account_ix],
            Some(&keypair.pubkey()),
            &[&keypair],
            recent_blockhash,
        );
        
        let signature = rpc_client
            .send_and_confirm_transaction(&transaction)
            .context("Failed to send transaction")?;
        
        // Verify account was created correctly
        self.verify_created_account(event).await?;
        
        debug!("Account created successfully: {}", signature);
        Ok(signature)
    }
    
    /// Verify that the created account matches expectations
    async fn verify_created_account(&self, event: &PDAComputedEvent) -> Result<()> {
        // Use valence-domain-client to verify account exists
        let account_exists = self.client.account_exists(&event.expected_pda.to_string()).await?;
        if !account_exists {
            return Err(SessionBuilderError::AccountVerificationFailed(
                "Account not found after creation".to_string(),
            ).into());
        }
        
        // For detailed verification, we still need the traditional client
        let rpc_client = solana_client::rpc_client::RpcClient::new_with_commitment(
            self.config.rpc_url.clone(),
            CommitmentConfig::confirmed(),
        );
        
        let account = rpc_client
            .get_account(&event.expected_pda)
            .context("Failed to fetch created account")?;
        
        // Verify size
        if account.data.len() != event.expected_size {
            return Err(SessionBuilderError::AccountVerificationFailed(
                format!(
                    "Size mismatch: expected {}, got {}",
                    event.expected_size,
                    account.data.len()
                ),
            ).into());
        }
        
        // Verify owner
        if account.owner != event.expected_owner {
            return Err(SessionBuilderError::AccountVerificationFailed(
                format!(
                    "Owner mismatch: expected {}, got {}",
                    event.expected_owner, account.owner
                ),
            ).into());
        }
        
        // Verify account is uninitialized (discriminator is zeros)
        if account.data.len() >= 8 {
            let discriminator = &account.data[0..8];
            if !discriminator.iter().all(|&b| b == 0) {
                return Err(SessionBuilderError::AccountVerificationFailed(
                    "Account appears to be already initialized".to_string(),
                ).into());
            }
        }
        
        debug!("Account verification passed for: {}", event.expected_pda);
        Ok(())
    }
    
    /// Start the health check background task
    fn start_health_check_task(&self) -> JoinHandle<()> {
        let client = self.client.clone();
        let is_running = self.is_running.clone();
        let interval = Duration::from_secs(self.config.monitoring.health_check_interval_secs);
        
        tokio::spawn(async move {
            while is_running.load(Ordering::Relaxed) {
                if let Err(e) = Self::health_check(&client).await {
                    error!("Health check failed: {}", e);
                }
                sleep(interval).await;
            }
        })
    }
    
    /// Perform health check
    async fn health_check(client: &C) -> Result<()> {
        // Check RPC connection using valence domain client
        let block_height = client.latest_block_height().await
            .context("Failed to get current block height")?;
        
        // Check wallet balance using valence domain client
        let balance = client.get_sol_balance_as_sol().await
            .context("Failed to get wallet balance")?;
        
        if balance == 0.0 {
            warn!("Service wallet has zero balance");
        }
        
        debug!("Health check passed - block height: {}, balance: {} SOL", block_height, balance);
        Ok(())
    }
    
    /// Verify initial setup
    async fn verify_setup(client: &C) -> Result<()> {
        // Test RPC connection using valence domain client
        let block_height = client.latest_block_height().await
            .context("Failed to connect to RPC")?;
        info!("Connected to Solana RPC at block height {}", block_height);
        
        // Check wallet balance using valence domain client
        let balance = client.get_sol_balance_as_sol().await
            .context("Failed to get balance")?;
        let address = client.signing_key_address();
        info!("Service wallet: {} (balance: {} SOL)", address, balance);
        
        if balance == 0.0 {
            warn!("Service wallet has no SOL balance - account creation may fail");
        }
        
        Ok(())
    }
    
    /// Get service statistics
    pub fn get_stats(&self) -> BTreeMap<String, u64> {
        let mut stats = BTreeMap::new();
        stats.insert("accounts_created".to_string(), self.accounts_created.load(Ordering::Relaxed));
        stats.insert("accounts_failed".to_string(), self.accounts_failed.load(Ordering::Relaxed));
        stats.insert("is_running".to_string(), self.is_running.load(Ordering::Relaxed) as u64);
        stats
    }
}

impl<C: SolanaRpcClient + SolanaSigningClient + SolanaBaseClient + Send + Sync + Clone> Clone for SessionBuilder<C> {
    fn clone(&self) -> Self {
        SessionBuilder {
            config: self.config.clone(),
            client: self.client.clone(),
            event_monitor: self.event_monitor.clone(),
            metrics: self.metrics.clone(),
            is_running: self.is_running.clone(),
            accounts_created: self.accounts_created.clone(),
            accounts_failed: self.accounts_failed.clone(),
            semaphore: self.semaphore.clone(),
        }
    }
}