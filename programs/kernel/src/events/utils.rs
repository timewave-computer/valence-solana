/// Event utility functions and helper methods
/// This module provides convenience functions for creating events and managing the event system
use anchor_lang::prelude::*;
use super::{
    EventManager, EventData, EventType,
    handlers::{
        SystemEventHandler, SingletonEventHandler, CapabilityEventHandler, 
        VerificationEventHandler, SessionEventHandler, FunctionEventHandler, RegistryEventHandler
    }
};

/// Create a default event manager with standard handlers
pub fn create_default_event_manager() -> EventManager {
    let mut manager = EventManager::new();
    
    // Add standard handlers
    manager.add_handler(Box::new(SystemEventHandler {
        handler_id: "system_handler".to_string(),
    }));
    
    manager.add_handler(Box::new(CapabilityEventHandler {
        handler_id: "capability_handler".to_string(),
    }));
    
    manager.add_handler(Box::new(VerificationEventHandler {
        handler_id: "verification_handler".to_string(),
    }));
    
    manager.add_handler(Box::new(SessionEventHandler {
        handler_id: "session_handler".to_string(),
    }));
    
    manager.add_handler(Box::new(FunctionEventHandler {
        handler_id: "function_handler".to_string(),
    }));
    
    manager.add_handler(Box::new(RegistryEventHandler {
        handler_id: "registry_handler".to_string(),
    }));
    
    // Add singleton handlers for each singleton type
    manager.add_handler(Box::new(SingletonEventHandler {
        handler_id: "processor_handler".to_string(),
        singleton_name: "processor".to_string(),
    }));
    
    manager.add_handler(Box::new(SingletonEventHandler {
        handler_id: "scheduler_handler".to_string(),
        singleton_name: "scheduler".to_string(),
    }));
    
    manager.add_handler(Box::new(SingletonEventHandler {
        handler_id: "diff_handler".to_string(),
        singleton_name: "diff".to_string(),
    }));
    
    manager
}

/// Helper function to emit a capability execution event
pub fn emit_capability_executed(
    manager: &EventManager,
    capability_id: String,
    shard: Pubkey,
    execution_time_ms: u64,
) -> Result<()> {
    let event = EventData::capability_event(
        EventType::CapabilityExecuted,
        capability_id,
        shard,
    ).with_metadata("execution_time_ms".to_string(), execution_time_ms.to_string());
    
    manager.emit_event(&event)
}

/// Helper function to emit a verification event
pub fn emit_verification_result(
    manager: &EventManager,
    verification_functions: Vec<String>,
    success: bool,
) -> Result<()> {
    let event_type = if success {
        EventType::VerificationCompleted
    } else {
        EventType::VerificationFailed
    };
    
    let event = EventData::verification_event(event_type, verification_functions);
    manager.emit_event(&event)
}

/// Helper function to emit a system state change event
pub fn emit_system_state_change(
    manager: &EventManager,
    is_paused: bool,
    reason: String,
) -> Result<()> {
    let event_type = if is_paused {
        EventType::SystemPaused
    } else {
        EventType::SystemResumed
    };
    
    let event = EventData::system_event(event_type, reason);
    manager.emit_event(&event)
}

/// Helper function to emit a singleton initialization event
pub fn emit_singleton_initialized(
    manager: &EventManager,
    singleton_name: String,
    config_details: String,
) -> Result<()> {
    let event_type = match singleton_name.as_str() {
        "processor" => EventType::ProcessorInitialized,
        "scheduler" => EventType::SchedulerInitialized,
        "diff" => EventType::DiffInitialized,
        _ => return Ok(()), // Unknown singleton type
    };
    
    let event = EventData::singleton_event(event_type, singleton_name, config_details);
    manager.emit_event(&event)
}

/// Helper function to emit a session lifecycle event
pub fn emit_session_lifecycle(
    manager: &EventManager,
    event_type: EventType,
    session_id: String,
    session_pubkey: Pubkey,
    additional_metadata: Option<(String, String)>,
) -> Result<()> {
    let mut event = EventData::session_event(event_type, session_id, session_pubkey);
    
    if let Some((key, value)) = additional_metadata {
        event = event.with_metadata(key, value);
    }
    
    manager.emit_event(&event)
}

/// Helper function to emit a function execution event
pub fn emit_function_execution(
    manager: &EventManager,
    event_type: EventType,
    function_hash: String,
    execution_context: String,
    execution_time_ms: u64,
) -> Result<()> {
    let event = EventData::function_event(event_type, function_hash, execution_context)
        .with_metadata("execution_time_ms".to_string(), execution_time_ms.to_string());
    
    manager.emit_event(&event)
}

/// Helper function to emit a registry operation event
pub fn emit_registry_operation(
    manager: &EventManager,
    event_type: EventType,
    item_name: String,
    item_version: String,
    operation_result: bool,
) -> Result<()> {
    let event = EventData::registry_event(event_type, item_name, item_version)
        .with_metadata("success".to_string(), operation_result.to_string());
    
    manager.emit_event(&event)
}

/// Helper function to emit a configuration change event
pub fn emit_configuration_updated(
    manager: &EventManager,
    config_type: String,
    changes: Vec<(String, String)>,
) -> Result<()> {
    let mut event = EventData::new(
        EventType::ConfigurationUpdated,
        format!("config:{config_type}"),
        vec![]
    ).with_metadata("category".to_string(), "configuration".to_string())
     .with_metadata("config_type".to_string(), config_type);
    
    // Add change details as metadata
    for (key, value) in changes {
        event = event.with_metadata(format!("change_{key}"), value);
    }
    
    manager.emit_event(&event)
}

/// Helper function to emit an error event with context
pub fn emit_error_with_context(
    manager: &EventManager,
    error_code: String,
    error_message: String,
    context: String,
    severity: ErrorSeverity,
) -> Result<()> {
    let event = EventData::error_event(error_code, error_message, context)
        .with_metadata("severity".to_string(), severity.to_string());
    
    manager.emit_event(&event)
}

/// Get event category from event type
pub fn get_event_category(event_type: &EventType) -> &'static str {
    match event_type {
        EventType::SystemInitialized | 
        EventType::SystemPaused | 
        EventType::SystemResumed => "system",
        
        EventType::ProcessorInitialized | EventType::ProcessorPaused | EventType::ProcessorResumed |
        EventType::CapabilityProcessed | EventType::SchedulerInitialized | EventType::ExecutionScheduled |
        EventType::QueueProcessed | EventType::ResourcesAllocated | EventType::PartialOrdersComposed |
        EventType::DiffInitialized | EventType::DiffCalculated | EventType::DiffProcessed |
        EventType::BatchOptimized => "singleton",
        
        EventType::CapabilityGranted | EventType::CapabilityUpdated | 
        EventType::CapabilityRevoked | EventType::CapabilityExecuted => "capability",
        
        EventType::SessionCreated | EventType::SessionActivated | 
        EventType::SessionClosed | EventType::SessionDataUpdated => "session",
        
        EventType::VerificationStarted | EventType::VerificationCompleted | 
        EventType::VerificationFailed => "verification",
        
        EventType::FunctionExecuted | EventType::FunctionChainExecuted | 
        EventType::FunctionAggregated => "function",
        
        EventType::LibraryRegistered | EventType::ZkProgramRegistered | 
        EventType::DependencyAdded => "registry",
        
        EventType::ConfigurationUpdated => "configuration",
        EventType::ErrorOccurred => "error",
    }
}

/// Check if an event type represents a critical system event
pub fn is_critical_event(event_type: &EventType) -> bool {
    matches!(event_type,
        EventType::SystemPaused |
        EventType::SystemResumed |
        EventType::ProcessorPaused |
        EventType::ProcessorResumed |
        EventType::VerificationFailed |
        EventType::ErrorOccurred
    )
}

/// Check if an event type represents a lifecycle event
pub fn is_lifecycle_event(event_type: &EventType) -> bool {
    matches!(event_type,
        EventType::SystemInitialized |
        EventType::ProcessorInitialized |
        EventType::SchedulerInitialized |
        EventType::DiffInitialized |
        EventType::SessionCreated |
        EventType::SessionActivated |
        EventType::SessionClosed |
        EventType::CapabilityGranted |
        EventType::CapabilityRevoked |
        EventType::LibraryRegistered |
        EventType::ZkProgramRegistered
    )
}

/// Error severity levels for error events
#[derive(Clone, Debug)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for ErrorSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorSeverity::Low => write!(f, "low"),
            ErrorSeverity::Medium => write!(f, "medium"),
            ErrorSeverity::High => write!(f, "high"),
            ErrorSeverity::Critical => write!(f, "critical"),
        }
    }
} 