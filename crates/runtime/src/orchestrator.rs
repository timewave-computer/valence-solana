//! Protocol flow orchestration without holding keys

use crate::{event_stream::EventStream, Result, RuntimeError};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};
use std::{collections::HashMap, sync::Arc, time::Duration};
use tokio::{
    sync::{broadcast, RwLock},
    task::JoinHandle,
    time::interval,
};
use tracing::{debug, info, warn};

/// Protocol flow definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolFlow {
    pub id: String,
    pub name: String,
    pub steps: Vec<FlowStep>,
    pub timeout: Duration,
    pub retry_policy: RetryPolicy,
}

/// Individual step in a protocol flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowStep {
    pub name: String,
    pub description: String,
    pub instructions: Vec<InstructionTemplate>,
    pub conditions: Vec<Condition>,
    pub on_success: Option<String>, // Next step ID
    pub on_failure: Option<String>, // Fallback step ID
}

/// Instruction template for building transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstructionTemplate {
    pub program_id: Pubkey,
    pub accounts: Vec<AccountTemplate>,
    pub data: Vec<u8>,
}

/// Account template for instruction building
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountTemplate {
    pub pubkey: Option<Pubkey>,
    pub is_signer: bool,
    pub is_writable: bool,
    pub derivation: Option<AccountDerivation>,
}

/// Account derivation info for PDAs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountDerivation {
    pub program_id: Pubkey,
    pub seeds: Vec<Vec<u8>>,
}

/// Flow execution condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Condition {
    AccountExists(Pubkey),
    AccountDataEquals {
        pubkey: Pubkey,
        offset: usize,
        data: Vec<u8>,
    },
    BalanceGreaterThan {
        pubkey: Pubkey,
        lamports: u64,
    },
    Custom(String), // Custom condition ID
}

/// Retry policy for failed steps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub delay: Duration,
    pub backoff_multiplier: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            delay: Duration::from_secs(1),
            backoff_multiplier: 2.0,
        }
    }
}

/// Flow execution state
#[derive(Debug, Clone)]
pub struct FlowExecution {
    pub flow_id: String,
    pub instance_id: String,
    pub current_step: String,
    pub status: ExecutionStatus,
    pub context: HashMap<String, serde_json::Value>,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Execution status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Pending,
    Running,
    WaitingForSignature,
    Completed,
    Failed(String),
}

/// Orchestrator for managing protocol flows
pub struct Orchestrator {
    rpc_client: Arc<RpcClient>,
    event_stream: Arc<EventStream>,
    flows: Arc<RwLock<HashMap<String, ProtocolFlow>>>,
    executions: Arc<DashMap<String, FlowExecution>>,
    shutdown_tx: broadcast::Sender<()>,
    worker_handle: Arc<RwLock<Option<JoinHandle<()>>>>,
}

impl Orchestrator {
    /// Create a new orchestrator
    pub fn new(rpc_client: Arc<RpcClient>, event_stream: Arc<EventStream>) -> Self {
        let (shutdown_tx, _) = broadcast::channel(16);

        Self {
            rpc_client,
            event_stream,
            flows: Arc::new(RwLock::new(HashMap::new())),
            executions: Arc::new(DashMap::new()),
            shutdown_tx,
            worker_handle: Arc::new(RwLock::new(None)),
        }
    }

    /// Start the orchestrator
    pub async fn start(&self) -> Result<()> {
        info!("Starting orchestrator");

        let _rpc_client = self.rpc_client.clone();
        let _event_stream = self.event_stream.clone();
        let _flows = self.flows.clone();
        let executions = self.executions.clone();
        let mut shutdown_rx = self.shutdown_tx.subscribe();

        let handle = tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(1));

            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        info!("Orchestrator received shutdown signal");
                        break;
                    }
                    _ = interval.tick() => {
                        // Process active executions
                        for execution in executions.iter() {
                            let exec = execution.value();
                            if matches!(exec.status, ExecutionStatus::Running) {
                                // Process execution step
                                debug!("Processing execution: {}", exec.instance_id);
                            }
                        }
                    }
                }
            }
        });

        *self.worker_handle.write().await = Some(handle);

        Ok(())
    }

    /// Stop the orchestrator
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping orchestrator");

        // Send shutdown signal
        let _ = self.shutdown_tx.send(());

        // Wait for worker to finish
        if let Some(handle) = self.worker_handle.write().await.take() {
            let _ = handle.await;
        }

        Ok(())
    }

    /// Register a protocol flow
    pub async fn register_flow(&self, flow: ProtocolFlow) -> Result<()> {
        info!("Registering flow: {}", flow.name);

        self.flows.write().await.insert(flow.id.clone(), flow);

        Ok(())
    }

    /// Start a flow execution
    pub async fn start_flow(
        &self,
        flow_id: String,
        context: HashMap<String, serde_json::Value>,
    ) -> Result<String> {
        let flows = self.flows.read().await;
        let flow = flows.get(&flow_id).ok_or_else(|| {
            RuntimeError::OrchestrationError(format!("Flow not found: {}", flow_id))
        })?;

        let instance_id = uuid::Uuid::new_v4().to_string();

        let execution = FlowExecution {
            flow_id: flow_id.clone(),
            instance_id: instance_id.clone(),
            current_step: flow.steps[0].name.clone(),
            status: ExecutionStatus::Pending,
            context,
            started_at: chrono::Utc::now(),
            completed_at: None,
        };

        self.executions.insert(instance_id.clone(), execution);

        // Emit event
        self.event_stream
            .emit(crate::event_stream::Event::FlowStarted {
                flow_id,
                instance_id: instance_id.clone(),
            })
            .await;

        Ok(instance_id)
    }

    /// Build instructions for a flow step
    pub async fn build_step_instructions(
        &self,
        flow_id: &str,
        step_name: &str,
        context: &HashMap<String, serde_json::Value>,
    ) -> Result<Vec<Instruction>> {
        let flows = self.flows.read().await;
        let flow = flows.get(flow_id).ok_or_else(|| {
            RuntimeError::OrchestrationError(format!("Flow not found: {}", flow_id))
        })?;

        let step = flow
            .steps
            .iter()
            .find(|s| s.name == step_name)
            .ok_or_else(|| {
                RuntimeError::OrchestrationError(format!("Step not found: {}", step_name))
            })?;

        let mut instructions = Vec::new();

        for template in &step.instructions {
            let instruction = self.build_instruction(template, context).await?;
            instructions.push(instruction);
        }

        Ok(instructions)
    }

    /// Build a single instruction from template
    async fn build_instruction(
        &self,
        template: &InstructionTemplate,
        _context: &HashMap<String, serde_json::Value>,
    ) -> Result<Instruction> {
        let mut accounts = Vec::new();

        for account_template in &template.accounts {
            let pubkey = if let Some(pubkey) = account_template.pubkey {
                pubkey
            } else if let Some(derivation) = &account_template.derivation {
                // Derive PDA
                let (pubkey, _) = Pubkey::find_program_address(
                    &derivation
                        .seeds
                        .iter()
                        .map(|s| s.as_slice())
                        .collect::<Vec<_>>(),
                    &derivation.program_id,
                );
                pubkey
            } else {
                // Look up in context
                return Err(RuntimeError::OrchestrationError(
                    "Account pubkey not found".to_string(),
                ));
            };

            accounts.push(solana_sdk::instruction::AccountMeta {
                pubkey,
                is_signer: account_template.is_signer,
                is_writable: account_template.is_writable,
            });
        }

        Ok(Instruction {
            program_id: template.program_id,
            accounts,
            data: template.data.clone(),
        })
    }

    /// Check if conditions are met
    pub async fn check_conditions(&self, conditions: &[Condition]) -> Result<bool> {
        for condition in conditions {
            match condition {
                Condition::AccountExists(pubkey) => {
                    let account = self.rpc_client.get_account(pubkey).await;
                    if account.is_err() {
                        return Ok(false);
                    }
                }
                Condition::AccountDataEquals {
                    pubkey,
                    offset,
                    data,
                } => {
                    let account = self.rpc_client.get_account(pubkey).await?;
                    let account_data = &account.data[*offset..(*offset + data.len())];
                    if account_data != data.as_slice() {
                        return Ok(false);
                    }
                }
                Condition::BalanceGreaterThan { pubkey, lamports } => {
                    let balance = self.rpc_client.get_balance(pubkey).await?;
                    if balance <= *lamports {
                        return Ok(false);
                    }
                }
                Condition::Custom(id) => {
                    // Custom condition logic would go here
                    warn!("Custom condition not implemented: {}", id);
                }
            }
        }

        Ok(true)
    }

    /// Get execution status
    pub async fn get_execution_status(&self, instance_id: &str) -> Option<FlowExecution> {
        self.executions.get(instance_id).map(|e| e.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_orchestrator_creation() {
        let rpc_client = Arc::new(RpcClient::new(
            "https://api.mainnet-beta.solana.com".to_string(),
        ));
        let event_stream = Arc::new(EventStream::new());
        let orchestrator = Orchestrator::new(rpc_client, event_stream);

        assert!(orchestrator.start().await.is_ok());
        assert!(orchestrator.stop().await.is_ok());
    }

    #[tokio::test]
    async fn test_flow_registration() {
        let rpc_client = Arc::new(RpcClient::new(
            "https://api.mainnet-beta.solana.com".to_string(),
        ));
        let event_stream = Arc::new(EventStream::new());
        let orchestrator = Orchestrator::new(rpc_client, event_stream);

        let flow = ProtocolFlow {
            id: "test-flow".to_string(),
            name: "Test Flow".to_string(),
            steps: vec![],
            timeout: Duration::from_secs(60),
            retry_policy: RetryPolicy::default(),
        };

        assert!(orchestrator.register_flow(flow).await.is_ok());
    }
}
