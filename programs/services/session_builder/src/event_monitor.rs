//! Event monitoring for PDAComputedEvent emissions

use anyhow::Result;
use borsh::{BorshDeserialize, BorshSerialize};
use futures::stream::Stream;
use solana_client::{
    nonblocking::pubsub_client::PubsubClient,
    rpc_client::RpcClient,
    rpc_config::{RpcTransactionLogsConfig, RpcTransactionLogsFilter},
};
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
};
use std::{
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    task::{Context as TaskContext, Poll},
};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::error::SessionBuilderError;

/// PDA Computed Event data structure (updated to match session_factory event)
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct PDAComputedEvent {
    /// The computed session PDA where account should be created
    pub expected_pda: Pubkey,
    /// Expected size of the session account
    pub expected_size: usize,
    /// Expected owner (session program)
    pub expected_owner: Pubkey,
    /// Session ID for initialization
    pub session_id: String,
    /// Session owner
    pub owner: Pubkey,
    /// Eval program bound to session
    pub eval_program: Pubkey,
    /// Shard this session is associated with
    pub shard_address: Pubkey,
    /// Namespaces for the session
    pub namespaces: Vec<String>,
    /// Block slot when computed
    pub compute_slot: u64,
}

/// Event monitor for tracking PDAComputedEvent emissions
#[derive(Clone)]
pub struct EventMonitor {
    rpc_client: Arc<RpcClient>,
    program_id: Pubkey,
    is_running: Arc<AtomicBool>,
}

impl EventMonitor {
    /// Create a new EventMonitor
    pub async fn new(rpc_client: Arc<RpcClient>, program_id: Pubkey) -> Result<Self> {
        Ok(EventMonitor {
            rpc_client,
            program_id,
            is_running: Arc::new(AtomicBool::new(false)),
        })
    }
    
    /// Start monitoring events
    pub async fn start(&self) -> Result<PDAEventStream> {
        if self.is_running.load(Ordering::Relaxed) {
            return Err(SessionBuilderError::AlreadyRunning.into());
        }
        
        self.is_running.store(true, Ordering::Relaxed);
        info!("Starting event monitoring for program: {}", self.program_id);
        
        let (tx, rx) = mpsc::unbounded_channel();
        
        // Start log monitoring in background task
        let monitor = self.clone();
        tokio::spawn(async move {
            if let Err(e) = monitor.monitor_logs(tx).await {
                error!("Log monitoring failed: {}", e);
            }
        });
        
        Ok(PDAEventStream::new(rx))
    }
    
    /// Stop monitoring events
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping event monitoring");
        self.is_running.store(false, Ordering::Relaxed);
        Ok(())
    }
    
    /// Monitor transaction logs for PDAComputedEvent emissions
    async fn monitor_logs(&self, sender: mpsc::UnboundedSender<Result<PDAComputedEvent>>) -> Result<()> {
        // Convert HTTP RPC URL to WebSocket URL
        let ws_url = self.rpc_client.url().replace("http", "ws");
        
        let pubsub_client = match PubsubClient::new(&ws_url).await {
            Ok(client) => client,
            Err(e) => {
                error!("Failed to connect to WebSocket: {}", e);
                return Err(e.into());
            }
        };
        
        let config = RpcTransactionLogsConfig {
            commitment: Some(CommitmentConfig::confirmed()),
        };
        
        let filter = RpcTransactionLogsFilter::Mentions(vec![self.program_id.to_string()]);
        
        let (mut logs_stream, logs_unsubscribe) = match pubsub_client
            .logs_subscribe(filter, config)
            .await
        {
            Ok(subscription) => subscription,
            Err(e) => {
                error!("Failed to subscribe to logs: {}", e);
                return Err(e.into());
            }
        };
        
        info!("Successfully subscribed to transaction logs");
        
        use futures::StreamExt;
        
        while self.is_running.load(Ordering::Relaxed) {
            match logs_stream.next().await {
                Some(log_result) => {
                    if let Err(e) = self.process_logs(&log_result.value.logs, &sender) {
                        warn!("Failed to process logs: {}", e);
                    }
                }
                None => {
                    error!("Log stream ended unexpectedly");
                    break;
                }
            }
        }
        
        // Cleanup
        logs_unsubscribe().await;
        
        Ok(())
    }
    
    /// Process transaction logs to extract PDAComputedEvent
    fn process_logs(
        &self,
        logs: &[String],
        sender: &mpsc::UnboundedSender<Result<PDAComputedEvent>>,
    ) -> Result<()> {
        for log in logs {
            if log.contains("PDAComputedEvent") {
                match self.parse_pda_computed_event(log) {
                    Ok(event) => {
                        debug!("Parsed PDAComputedEvent: {:?}", event);
                        if let Err(e) = sender.send(Ok(event)) {
                            error!("Failed to send event to channel: {}", e);
                        }
                    }
                    Err(e) => {
                        warn!("Failed to parse PDAComputedEvent from log: {}", e);
                        if let Err(send_err) = sender.send(Err(e)) {
                            error!("Failed to send error to channel: {}", send_err);
                        }
                    }
                }
            }
        }
        Ok(())
    }
    
    /// Parse PDAComputedEvent from log entry
    fn parse_pda_computed_event(&self, log: &str) -> Result<PDAComputedEvent> {
        // Look for the event data in the log
        // The actual implementation would depend on how Anchor emits events
        // This is a simplified version - in reality you'd parse the base64 encoded event data
        
        // For now, we'll implement a basic parser that looks for specific patterns
        // In a production system, you'd want to use proper Anchor event parsing
        
        if let Some(data_start) = log.find("data: ") {
            let data_str = &log[data_start + 6..];
            if let Some(data_end) = data_str.find(' ') {
                let data_str = &data_str[..data_end];
                
                // Try to decode base64 data
                if let Ok(data_bytes) = bs58::decode(data_str).into_vec() {
                    // Try to deserialize as PDAComputedEvent
                    if let Ok(event) = PDAComputedEvent::try_from_slice(&data_bytes) {
                        return Ok(event);
                    }
                }
            }
        }
        
        Err(SessionBuilderError::EventParsingError(
            "Could not parse PDAComputedEvent from log".to_string(),
        ).into())
    }
}

/// Stream of PDA computed events
pub struct PDAEventStream {
    receiver: mpsc::UnboundedReceiver<Result<PDAComputedEvent>>,
}

impl PDAEventStream {
    fn new(receiver: mpsc::UnboundedReceiver<Result<PDAComputedEvent>>) -> Self {
        Self { receiver }
    }
}

impl Stream for PDAEventStream {
    type Item = Result<PDAComputedEvent>;
    
    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut TaskContext<'_>,
    ) -> Poll<Option<Self::Item>> {
        match self.receiver.poll_recv(cx) {
            Poll::Ready(Some(item)) => Poll::Ready(Some(item)),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
} 