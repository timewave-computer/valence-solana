//! Protocol flow coordination and execution

use crate::{monitoring::event_stream::EventStream, Result, RuntimeError};
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
use tracing::{debug, info};

// ================================
// Protocol Flow Types
// ================================

/// Simplified protocol flow definition aligned with kernel instructions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolFlow {
    pub id: String,
    pub name: String,
    pub steps: Vec<FlowStep>,
    pub timeout: Duration,
    pub retry_policy: RetryPolicy,
}

/// Individual step in a protocol flow (simplified)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowStep {
    pub name: String,
    pub description: String,
    pub instruction_type: KernelInstructionType,
    pub on_success: Option<String>, // Next step ID
    pub on_failure: Option<String>, // Fallback step ID
}

/// Kernel instruction types that align with actual valence-kernel instructions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KernelInstructionType {
    /// Initialize shard
    InitializeShard,
    /// Create guard account
    CreateGuardAccount {
        session: Pubkey,
        allow_unregistered_cpi: bool,
    },
    /// Create session account with ALT
    CreateSessionAccount {
        shard: Pubkey,
        namespace: String,
        metadata: Vec<u8>,
    },
    /// Manage account lookup table
    ManageALT {
        add_accounts: Vec<Pubkey>,
        remove_accounts: Vec<Pubkey>,
    },
    /// Execute batch operations
    ExecuteBatch {
        accounts: Vec<Pubkey>,
        operations: Vec<BatchOperationType>,
    },
    /// Initialize CPI allowlist
    InitializeAllowlist,
    /// Add program to allowlist
    AddProgramToAllowlist { program_id: Pubkey },
}

/// Simplified batch operation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BatchOperationType {
    BorrowAccount { account_index: u8, mode: u8 },
    ReleaseAccount { account_index: u8 },
    CallRegisteredFunction { registry_id: u64, data: Vec<u8> },
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

// ================================
// Execution Types
// ================================

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

// ================================
// Coordinator Implementation
// ================================

/// Simplified coordinator for managing protocol flows
pub struct Coordinator {
    #[allow(dead_code)]
    rpc_client: Arc<RpcClient>,
    event_stream: Arc<EventStream>,
    flows: Arc<RwLock<HashMap<String, ProtocolFlow>>>,
    executions: Arc<DashMap<String, FlowExecution>>,
    shutdown_tx: broadcast::Sender<()>,
    worker_handle: Arc<RwLock<Option<JoinHandle<()>>>>,
}

impl Coordinator {
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
            RuntimeError::CoordinationError(format!("Flow not found: {}", flow_id))
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
            .emit(crate::monitoring::event_stream::Event::FlowStarted {
                flow_id,
                instance_id: instance_id.clone(),
            })
            .await;

        Ok(instance_id)
    }

    /// Build instruction for a flow step using actual kernel instructions
    pub async fn build_step_instruction(
        &self,
        flow_id: &str,
        step_name: &str,
        context: &HashMap<String, serde_json::Value>,
    ) -> Result<Instruction> {
        let flows = self.flows.read().await;
        let flow = flows.get(flow_id).ok_or_else(|| {
            RuntimeError::CoordinationError(format!("Flow not found: {}", flow_id))
        })?;

        let step = flow
            .steps
            .iter()
            .find(|s| s.name == step_name)
            .ok_or_else(|| {
                RuntimeError::CoordinationError(format!("Step not found: {}", step_name))
            })?;

        self.build_kernel_instruction(&step.instruction_type, context).await
    }

    /// Build actual kernel instruction
    async fn build_kernel_instruction(
        &self,
        instruction_type: &KernelInstructionType,
        _context: &HashMap<String, serde_json::Value>,
    ) -> Result<Instruction> {
        // TODO: Build actual kernel instructions using valence_kernel crate
        // For now, return placeholder instructions
        
        match instruction_type {
            KernelInstructionType::InitializeShard => {
                // Would use: valence_kernel::instructions::initialize_shard
                Ok(Instruction {
                    program_id: valence_kernel_program_id(),
                    accounts: vec![], // TODO: Add proper accounts
                    data: vec![0], // TODO: Add proper instruction data
                })
            }
            
            KernelInstructionType::CreateGuardAccount { session: _, allow_unregistered_cpi: _ } => {
                // Would use: valence_kernel::instructions::create_guard_account
                Ok(Instruction {
                    program_id: valence_kernel_program_id(),
                    accounts: vec![], // TODO: Add proper accounts
                    data: vec![1], // TODO: Add proper instruction data with session and flag
                })
            }
            
            KernelInstructionType::CreateSessionAccount { shard: _, namespace: _, metadata: _ } => {  
                // Would use: valence_kernel::instructions::create_session_account
                Ok(Instruction {
                    program_id: valence_kernel_program_id(),
                    accounts: vec![], // TODO: Add proper accounts
                    data: vec![2], // TODO: Add proper instruction data
                })
            }
            
            KernelInstructionType::ExecuteBatch { accounts: _, operations: _ } => {
                // Would use: valence_kernel::instructions::execute_batch
                Ok(Instruction {
                    program_id: valence_kernel_program_id(),
                    accounts: vec![], // TODO: Add proper accounts
                    data: vec![3], // TODO: Add proper batch data
                })
            }
            
            _ => {
                // TODO: Implement other instruction types
                Err(RuntimeError::CoordinationError(
                    "Instruction type not yet implemented".to_string(),
                ))
            }
        }
    }

    /// Get execution status
    pub async fn get_execution_status(&self, instance_id: &str) -> Option<FlowExecution> {
        self.executions.get(instance_id).map(|e| e.clone())
    }
}

/// Get the kernel program ID (placeholder)
fn valence_kernel_program_id() -> Pubkey {
    // This should be the actual valence-kernel program ID
    // Using hardcoded value to avoid anchor_lang dependency
    "Va1ence111111111111111111111111111111111111".parse().unwrap()
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
        let coordinator = Coordinator::new(rpc_client, event_stream);

        assert!(coordinator.start().await.is_ok());
        assert!(coordinator.stop().await.is_ok());
    }

    #[tokio::test]
    async fn test_flow_registration() {
        let rpc_client = Arc::new(RpcClient::new(
            "https://api.mainnet-beta.solana.com".to_string(),
        ));
        let event_stream = Arc::new(EventStream::new());
        let coordinator = Coordinator::new(rpc_client, event_stream);

        let flow = ProtocolFlow {
            id: "test-flow".to_string(),
            name: "Test Flow".to_string(),
            steps: vec![FlowStep {
                name: "init_shard".to_string(),
                description: "Initialize shard".to_string(),
                instruction_type: KernelInstructionType::InitializeShard,
                on_success: None,
                on_failure: None,
            }],
            timeout: Duration::from_secs(60),
            retry_policy: RetryPolicy::default(),
        };

        assert!(coordinator.register_flow(flow).await.is_ok());
    }
}