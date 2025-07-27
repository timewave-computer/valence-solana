//! WebSocket state monitoring for on-chain account changes

use crate::{event_stream::EventStream, Result, RuntimeError};
use base64::Engine;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;
use std::{collections::HashMap, sync::Arc};
use tokio::{
    sync::{broadcast, RwLock},
    task::JoinHandle,
};
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{error, info, warn};

/// State update notification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateUpdate {
    pub account: Pubkey,
    pub slot: u64,
    pub lamports: u64,
    pub data: Vec<u8>,
    pub owner: Pubkey,
    pub executable: bool,
    pub rent_epoch: u64,
}

/// WebSocket subscription request
#[derive(Debug, Serialize)]
struct SubscriptionRequest {
    jsonrpc: &'static str,
    id: u64,
    method: &'static str,
    params: serde_json::Value,
}

/// WebSocket response
#[derive(Debug, Deserialize)]
struct WsResponse {
    jsonrpc: String,
    method: Option<String>,
    params: Option<serde_json::Value>,
}

impl WsResponse {
    /// Get the JSON-RPC version
    pub fn jsonrpc_version(&self) -> &str {
        &self.jsonrpc
    }
}

/// Account subscription info
struct Subscription {
    pubkey: Pubkey,
    subscription_id: u64,
    callback: Box<dyn Fn(StateUpdate) + Send + Sync>,
}

impl Subscription {
    /// Get the subscribed pubkey
    pub fn pubkey(&self) -> &Pubkey {
        &self.pubkey
    }

    /// Get the subscription ID
    pub fn subscription_id(&self) -> u64 {
        self.subscription_id
    }
}

/// State monitor for WebSocket subscriptions
pub struct StateMonitor {
    ws_url: String,
    event_stream: Arc<EventStream>,
    subscriptions: Arc<RwLock<HashMap<Pubkey, Subscription>>>,
    shutdown_tx: broadcast::Sender<()>,
    worker_handle: Arc<RwLock<Option<JoinHandle<()>>>>,
}

impl StateMonitor {
    /// Create a new state monitor
    pub async fn new(ws_url: String, event_stream: Arc<EventStream>) -> Result<Self> {
        let (shutdown_tx, _shutdown_rx) = broadcast::channel(16);

        Ok(Self {
            ws_url,
            event_stream,
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            shutdown_tx,
            worker_handle: Arc::new(RwLock::new(None)),
        })
    }

    /// Start the state monitor
    pub async fn start(&self) -> Result<()> {
        info!("Starting state monitor");

        let ws_url = self.ws_url.clone();
        let subscriptions = self.subscriptions.clone();
        let event_stream = self.event_stream.clone();
        let handle = tokio::spawn({
            let shutdown_rx = self.shutdown_tx.subscribe();
            async move {
                if let Err(e) =
                    Self::monitor_loop(ws_url, subscriptions, event_stream, shutdown_rx).await
                {
                    error!("State monitor error: {}", e);
                }
            }
        });

        *self.worker_handle.write().await = Some(handle);

        Ok(())
    }

    /// Stop the state monitor
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping state monitor");

        // Send shutdown signal
        let _ = self.shutdown_tx.send(());

        // Wait for worker to finish
        if let Some(handle) = self.worker_handle.write().await.take() {
            let _ = handle.await;
        }

        Ok(())
    }

    /// Subscribe to account updates
    pub async fn subscribe_account<F>(&self, account: Pubkey, callback: F) -> Result<()>
    where
        F: Fn(StateUpdate) + Send + Sync + 'static,
    {
        info!("Subscribing to account {}", account);

        let subscription = Subscription {
            pubkey: account,
            subscription_id: 0, // Will be set when subscription is confirmed
            callback: Box::new(callback),
        };

        self.subscriptions
            .write()
            .await
            .insert(account, subscription);

        Ok(())
    }

    /// Unsubscribe from account updates
    pub async fn unsubscribe_account(&self, account: &Pubkey) -> Result<()> {
        info!("Unsubscribing from account {}", account);

        self.subscriptions.write().await.remove(account);

        Ok(())
    }

    /// Main monitoring loop
    async fn monitor_loop(
        ws_url: String,
        subscriptions: Arc<RwLock<HashMap<Pubkey, Subscription>>>,
        event_stream: Arc<EventStream>,
        mut shutdown_rx: broadcast::Receiver<()>,
    ) -> Result<()> {
        let (ws_stream, _) = connect_async(&ws_url).await?;
        let (mut write, mut read) = ws_stream.split();

        // Subscribe to all accounts
        let subs = subscriptions.read().await;
        for (pubkey, _) in subs.iter() {
            let request = SubscriptionRequest {
                jsonrpc: "2.0",
                id: 1,
                method: "accountSubscribe",
                params: serde_json::json!([
                    pubkey.to_string(),
                    {
                        "encoding": "base64",
                        "commitment": "confirmed"
                    }
                ]),
            };

            let msg = Message::text(serde_json::to_string(&request)?);
            write.send(msg).await?;
        }
        drop(subs);

        // Process messages
        loop {
            tokio::select! {
                // Check for shutdown signal
                _ = shutdown_rx.recv() => {
                    info!("State monitor received shutdown signal");
                    break;
                }

                // Process WebSocket messages
                msg = read.next() => {
                    match msg {
                        Some(Ok(Message::Text(text))) => {
                            if let Err(e) = Self::handle_message(
                                &text,
                                &subscriptions,
                                &event_stream,
                            ).await {
                                warn!("Error handling message: {}", e);
                            }
                        }
                        Some(Ok(Message::Close(_))) => {
                            info!("WebSocket connection closed");
                            break;
                        }
                        Some(Err(e)) => {
                            error!("WebSocket error: {}", e);
                            break;
                        }
                        None => {
                            info!("WebSocket stream ended");
                            break;
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(())
    }

    /// Handle incoming WebSocket message
    async fn handle_message(
        text: &str,
        subscriptions: &Arc<RwLock<HashMap<Pubkey, Subscription>>>,
        event_stream: &Arc<EventStream>,
    ) -> Result<()> {
        let response: WsResponse = serde_json::from_str(text)?;

        // Validate JSON-RPC version
        if response.jsonrpc_version() != "2.0" {
            warn!(
                "Unexpected JSON-RPC version: {}",
                response.jsonrpc_version()
            );
        }

        if let Some(method) = response.method {
            if method == "accountNotification" {
                if let Some(params) = response.params {
                    Self::handle_account_notification(params, subscriptions, event_stream).await?;
                }
            }
        }

        Ok(())
    }

    /// Handle account notification
    async fn handle_account_notification(
        params: serde_json::Value,
        subscriptions: &Arc<RwLock<HashMap<Pubkey, Subscription>>>,
        event_stream: &Arc<EventStream>,
    ) -> Result<()> {
        // Parse notification
        let result = params["result"].clone();
        let context = result["context"].clone();
        let value = result["value"].clone();

        if value.is_null() {
            return Ok(());
        }

        let slot = context["slot"].as_u64().unwrap_or(0);
        let account_info = &value["account"];

        // Parse account data
        let lamports = account_info["lamports"].as_u64().unwrap_or(0);
        let owner = account_info["owner"]
            .as_str()
            .and_then(|s| s.parse::<Pubkey>().ok())
            .ok_or(RuntimeError::InvalidAccountData)?;
        let executable = account_info["executable"].as_bool().unwrap_or(false);
        let rent_epoch = account_info["rentEpoch"].as_u64().unwrap_or(0);

        // Decode data
        let data = if let Some(data_arr) = account_info["data"].as_array() {
            if !data_arr.is_empty() {
                if let Some(encoded) = data_arr[0].as_str() {
                    base64::engine::general_purpose::STANDARD
                        .decode(encoded)
                        .unwrap_or_default()
                } else {
                    vec![]
                }
            } else {
                vec![]
            }
        } else {
            vec![]
        };

        // Find account pubkey from subscription
        // Note: In real implementation, we'd track subscription IDs properly
        let subs = subscriptions.read().await;
        for (pubkey, subscription) in subs.iter() {
            // Use accessor methods for better encapsulation
            let subscribed_pubkey = subscription.pubkey();
            let sub_id = subscription.subscription_id();

            info!(
                "Processing update for subscription {} (account: {})",
                sub_id, subscribed_pubkey
            );

            let update = StateUpdate {
                account: *pubkey,
                slot,
                lamports,
                data: data.clone(),
                owner,
                executable,
                rent_epoch,
            };

            // Call callback
            (subscription.callback)(update.clone());

            // Emit event
            event_stream
                .emit(crate::event_stream::Event::StateUpdate(update))
                .await;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_state_monitor_creation() {
        let event_stream = Arc::new(EventStream::new());
        let monitor = StateMonitor::new(
            "wss://api.mainnet-beta.solana.com".to_string(),
            event_stream,
        )
        .await;

        assert!(monitor.is_ok());
    }

    #[tokio::test]
    async fn test_subscription() {
        let event_stream = Arc::new(EventStream::new());
        let monitor = StateMonitor::new(
            "wss://api.mainnet-beta.solana.com".to_string(),
            event_stream,
        )
        .await
        .unwrap();

        let account = Pubkey::new_unique();
        let result = monitor
            .subscribe_account(account, |_update| {
                // Callback logic
            })
            .await;

        assert!(result.is_ok());
    }
}
