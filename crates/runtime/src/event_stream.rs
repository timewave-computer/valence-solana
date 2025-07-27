//! State change event streaming

use crate::state_monitor::StateUpdate;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, error};

/// Runtime event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Event {
    /// State update from on-chain monitoring
    StateUpdate(StateUpdate),

    /// Flow execution started
    FlowStarted {
        flow_id: String,
        instance_id: String,
    },

    /// Flow step completed
    FlowStepCompleted {
        instance_id: String,
        step_name: String,
        success: bool,
    },

    /// Flow execution completed
    FlowCompleted {
        instance_id: String,
        success: bool,
        duration_ms: u64,
    },

    /// Transaction built
    TransactionBuilt {
        description: String,
        signers: Vec<solana_sdk::pubkey::Pubkey>,
        compute_units: Option<u32>,
    },

    /// Transaction submitted
    TransactionSubmitted {
        signature: String,
        description: String,
    },

    /// Transaction confirmed
    TransactionConfirmed {
        signature: String,
        slot: u64,
        error: Option<String>,
    },

    /// Audit log entry
    AuditLog {
        operation: String,
        details: serde_json::Value,
    },

    /// Error occurred
    Error { context: String, error: String },

    /// Warning
    Warning { context: String, message: String },
}

/// Event stream for broadcasting runtime events
pub struct EventStream {
    sender: broadcast::Sender<Event>,
    receiver_count: Arc<RwLock<usize>>,
}

impl EventStream {
    /// Create a new event stream
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1000); // Buffer up to 1000 events

        Self {
            sender,
            receiver_count: Arc::new(RwLock::new(0)),
        }
    }

    /// Subscribe to events
    pub async fn subscribe(&self) -> broadcast::Receiver<Event> {
        *self.receiver_count.write().await += 1;
        self.sender.subscribe()
    }

    /// Emit an event
    pub async fn emit(&self, event: Event) {
        debug!("Emitting event: {:?}", event);

        match self.sender.send(event) {
            Ok(count) => {
                debug!("Event sent to {} receivers", count);
            }
            Err(e) => {
                // No receivers, event is dropped
                debug!("No receivers for event: {:?}", e);
            }
        }
    }

    /// Get current receiver count
    pub async fn receiver_count(&self) -> usize {
        *self.receiver_count.read().await
    }
}

impl Default for EventStream {
    fn default() -> Self {
        Self::new()
    }
}

/// Event filter for selective subscription
#[derive(Debug, Clone)]
pub struct EventFilter {
    pub include_state_updates: bool,
    pub include_flow_events: bool,
    pub include_transaction_events: bool,
    pub include_audit_logs: bool,
    pub include_errors: bool,
    pub account_filter: Option<Vec<solana_sdk::pubkey::Pubkey>>,
}

impl Default for EventFilter {
    fn default() -> Self {
        Self {
            include_state_updates: true,
            include_flow_events: true,
            include_transaction_events: true,
            include_audit_logs: true,
            include_errors: true,
            account_filter: None,
        }
    }
}

impl EventFilter {
    /// Check if an event passes the filter
    pub fn matches(&self, event: &Event) -> bool {
        match event {
            Event::StateUpdate(update) => {
                if !self.include_state_updates {
                    return false;
                }
                if let Some(accounts) = &self.account_filter {
                    accounts.contains(&update.account)
                } else {
                    true
                }
            }
            Event::FlowStarted { .. }
            | Event::FlowStepCompleted { .. }
            | Event::FlowCompleted { .. } => self.include_flow_events,

            Event::TransactionBuilt { .. }
            | Event::TransactionSubmitted { .. }
            | Event::TransactionConfirmed { .. } => self.include_transaction_events,

            Event::AuditLog { .. } => self.include_audit_logs,

            Event::Error { .. } | Event::Warning { .. } => self.include_errors,
        }
    }
}

/// Filtered event stream wrapper
pub struct FilteredEventStream {
    receiver: broadcast::Receiver<Event>,
    filter: EventFilter,
}

impl FilteredEventStream {
    /// Create a filtered event stream
    pub fn new(receiver: broadcast::Receiver<Event>, filter: EventFilter) -> Self {
        Self { receiver, filter }
    }

    /// Receive next filtered event
    pub async fn recv(&mut self) -> Option<Event> {
        loop {
            match self.receiver.recv().await {
                Ok(event) => {
                    if self.filter.matches(&event) {
                        return Some(event);
                    }
                    // Continue to next event if filtered out
                }
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    error!("Event stream lagged by {} events", n);
                    // Continue receiving
                }
                Err(broadcast::error::RecvError::Closed) => {
                    return None;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::pubkey::Pubkey;

    #[tokio::test]
    async fn test_event_stream() {
        let stream = EventStream::new();
        let mut receiver = stream.subscribe().await;

        // Emit event
        stream
            .emit(Event::Warning {
                context: "test".to_string(),
                message: "test warning".to_string(),
            })
            .await;

        // Receive event
        let event = receiver.recv().await;
        assert!(event.is_ok());

        match event.unwrap() {
            Event::Warning { context, message } => {
                assert_eq!(context, "test");
                assert_eq!(message, "test warning");
            }
            _ => panic!("Unexpected event type"),
        }
    }

    #[tokio::test]
    async fn test_event_filter() {
        let mut filter = EventFilter::default();
        filter.include_errors = false;

        let error_event = Event::Error {
            context: "test".to_string(),
            error: "test error".to_string(),
        };

        assert!(!filter.matches(&error_event));

        let warning_event = Event::Warning {
            context: "test".to_string(),
            message: "test warning".to_string(),
        };

        assert!(filter.matches(&warning_event));
    }

    #[tokio::test]
    async fn test_account_filter() {
        let account1 = Pubkey::new_unique();
        let account2 = Pubkey::new_unique();

        let mut filter = EventFilter::default();
        filter.account_filter = Some(vec![account1]);

        let update1 = Event::StateUpdate(StateUpdate {
            account: account1,
            slot: 100,
            lamports: 1000,
            data: vec![],
            owner: Pubkey::new_unique(),
            executable: false,
            rent_epoch: 0,
        });

        let update2 = Event::StateUpdate(StateUpdate {
            account: account2,
            slot: 100,
            lamports: 1000,
            data: vec![],
            owner: Pubkey::new_unique(),
            executable: false,
            rent_epoch: 0,
        });

        assert!(filter.matches(&update1));
        assert!(!filter.matches(&update2));
    }
}
