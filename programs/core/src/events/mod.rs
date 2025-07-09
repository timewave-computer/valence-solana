/// Event system and coordination for Valence Protocol
/// This module provides event handling, coordination, and messaging capabilities

use anchor_lang::prelude::*;

// Module declarations
pub mod handlers;
pub mod utils;

// Re-export common types and functions
pub use handlers::*;
pub use utils::*;

/// Event types supported by the system
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum EventType {
    // ==================== Core System Events ====================
    /// System lifecycle events
    SystemInitialized,
    SystemPaused,
    SystemResumed,
    
    // ==================== Singleton Events ====================
    /// Processor singleton events
    ProcessorInitialized,
    ProcessorPaused,
    ProcessorResumed,
    CapabilityProcessed,
    
    /// Scheduler singleton events
    SchedulerInitialized,
    ExecutionScheduled,
    QueueProcessed,
    ResourcesAllocated,
    PartialOrdersComposed,
    
    /// Diff singleton events
    DiffInitialized,
    DiffCalculated,
    DiffProcessed,
    BatchOptimized,
    
    // ==================== Capability Events ====================
    /// Capability lifecycle and execution
    CapabilityGranted,
    CapabilityUpdated,
    CapabilityRevoked,
    CapabilityExecuted,
    
    // ==================== Session Events ====================
    /// Session lifecycle events
    SessionCreated,
    SessionActivated,
    SessionClosed,
    SessionDataUpdated,
    
    // ==================== Verification Events ====================
    /// Verification process events
    VerificationStarted,
    VerificationCompleted,
    VerificationFailed,
    
    // ==================== Function Events ====================
    /// Function composition and execution
    FunctionExecuted,
    FunctionChainExecuted,
    FunctionAggregated,
    
    // ==================== Registry Events ====================
    /// Registry and library management
    LibraryRegistered,
    ZkProgramRegistered,
    DependencyAdded,
    
    // ==================== Configuration Events ====================
    /// System configuration changes
    ConfigurationUpdated,
    
    // ==================== Error Events ====================
    /// Error and exception events
    ErrorOccurred,
}

/// Event data structure
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct EventData {
    /// Event type
    pub event_type: EventType,
    /// Timestamp when event occurred
    pub timestamp: i64,
    /// Source of the event
    pub source: String,
    /// Event payload data
    pub payload: Vec<u8>,
    /// Event metadata
    pub metadata: Vec<(String, String)>,
}

impl EventData {
    /// Create a new event
    pub fn new(event_type: EventType, source: String, payload: Vec<u8>) -> Self {
        let clock = Clock::get().unwrap_or_default();
        Self {
            event_type,
            timestamp: clock.unix_timestamp,
            source,
            payload,
            metadata: vec![],
        }
    }
    
    /// Add metadata to the event
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.push((key, value));
        self
    }
    
    /// Create a system lifecycle event
    pub fn system_event(event_type: EventType, message: String) -> Self {
        Self::new(event_type, "system".to_string(), message.into_bytes())
            .with_metadata("category".to_string(), "system".to_string())
    }
    
    /// Create a singleton event (processor, scheduler, diff)
    pub fn singleton_event(event_type: EventType, singleton_name: String, details: String) -> Self {
        Self::new(event_type, singleton_name.clone(), details.into_bytes())
            .with_metadata("category".to_string(), "singleton".to_string())
            .with_metadata("singleton".to_string(), singleton_name)
    }
    
    /// Create a capability event
    pub fn capability_event(event_type: EventType, capability_id: String, shard: Pubkey) -> Self {
        Self::new(event_type, format!("capability:{}", capability_id), vec![])
            .with_metadata("category".to_string(), "capability".to_string())
            .with_metadata("capability_id".to_string(), capability_id)
            .with_metadata("shard".to_string(), shard.to_string())
    }
    
    /// Create a session event
    pub fn session_event(event_type: EventType, session_id: String, session_pubkey: Pubkey) -> Self {
        Self::new(event_type, format!("session:{}", session_id), vec![])
            .with_metadata("category".to_string(), "session".to_string())
            .with_metadata("session_id".to_string(), session_id)
            .with_metadata("session_pubkey".to_string(), session_pubkey.to_string())
    }
    
    /// Create a verification event
    pub fn verification_event(event_type: EventType, verification_functions: Vec<String>) -> Self {
        let payload = verification_functions.join(",").into_bytes();
        Self::new(event_type, "verification".to_string(), payload)
            .with_metadata("category".to_string(), "verification".to_string())
            .with_metadata("function_count".to_string(), verification_functions.len().to_string())
    }
    
    /// Create a function execution event
    pub fn function_event(event_type: EventType, function_hash: String, execution_context: String) -> Self {
        Self::new(event_type, format!("function:{}", function_hash), execution_context.into_bytes())
            .with_metadata("category".to_string(), "function".to_string())
            .with_metadata("function_hash".to_string(), function_hash)
    }
    
    /// Create a registry event
    pub fn registry_event(event_type: EventType, item_name: String, item_version: String) -> Self {
        Self::new(event_type, format!("registry:{}", item_name), vec![])
            .with_metadata("category".to_string(), "registry".to_string())
            .with_metadata("item_name".to_string(), item_name)
            .with_metadata("item_version".to_string(), item_version)
    }
    
    /// Create an error event
    pub fn error_event(error_code: String, error_message: String, context: String) -> Self {
        Self::new(EventType::ErrorOccurred, format!("error:{}", error_code), error_message.into_bytes())
            .with_metadata("category".to_string(), "error".to_string())
            .with_metadata("error_code".to_string(), error_code)
            .with_metadata("context".to_string(), context)
    }
}

/// Event handler trait
pub trait EventHandler {
    /// Handle an event
    fn handle_event(&self, event: &EventData) -> Result<()>;
    
    /// Get the event types this handler supports
    fn supported_event_types(&self) -> Vec<EventType>;
    
    /// Get handler priority (lower number = higher priority)
    fn priority(&self) -> u32 {
        100
    }
}

/// Basic event handler implementation
pub struct BasicEventHandler {
    pub handler_id: String,
    pub supported_types: Vec<EventType>,
}

impl EventHandler for BasicEventHandler {
    fn handle_event(&self, event: &EventData) -> Result<()> {
        msg!(
            "Handler {} processing event {:?} from {} at {}",
            self.handler_id,
            event.event_type,
            event.source,
            event.timestamp
        );
        Ok(())
    }
    
    fn supported_event_types(&self) -> Vec<EventType> {
        self.supported_types.clone()
    }
}

/// Event manager for coordinating events
pub struct EventManager {
    handlers: Vec<Box<dyn EventHandler>>,
}

impl EventManager {
    /// Create a new event manager
    pub fn new() -> Self {
        Self {
            handlers: vec![],
        }
    }
    
    /// Add an event handler
    pub fn add_handler(&mut self, handler: Box<dyn EventHandler>) {
        self.handlers.push(handler);
        // Sort by priority
        self.handlers.sort_by_key(|h| h.priority());
    }
    
    /// Emit an event to all relevant handlers
    pub fn emit_event(&self, event: &EventData) -> Result<()> {
        for handler in &self.handlers {
            if handler.supported_event_types().contains(&event.event_type) {
                handler.handle_event(event)?;
            }
        }
        Ok(())
    }
    
    /// Emit an event and filter by category
    pub fn emit_event_by_category(&self, event: &EventData, category: &str) -> Result<()> {
        // Check if event matches category
        let event_category = event.metadata.iter()
            .find(|(key, _)| key == "category")
            .map(|(_, value)| value.as_str())
            .unwrap_or("unknown");
            
        if event_category == category {
            self.emit_event(event)?;
        }
        Ok(())
    }
    
    /// Get handlers for a specific event type
    pub fn get_handlers_for_type(&self, event_type: &EventType) -> Vec<&Box<dyn EventHandler>> {
        self.handlers.iter()
            .filter(|handler| handler.supported_event_types().contains(event_type))
            .collect()
    }
}

/// Event state for on-chain event storage
#[account]
pub struct EventState {
    /// Event data
    pub event_data: EventData,
    /// Event ID
    pub event_id: String,
    /// Whether event was processed
    pub processed: bool,
    /// PDA bump
    pub bump: u8,
}

impl EventState {
    pub fn get_space(event_id: &str, payload_size: usize) -> usize {
        8 + // discriminator
        4 + event_id.len() + // event_id
        1 + // event_type enum
        8 + // timestamp
        4 + 50 + // source (assume max 50 chars)
        4 + payload_size + // payload
        4 + (100 * 2 * 50) + // metadata (assume max 100 pairs, 50 chars each)
        1 + // processed
        1   // bump
    }
}
