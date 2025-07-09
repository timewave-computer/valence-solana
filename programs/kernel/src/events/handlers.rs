/// Specialized event handlers for different categories of events
/// This module contains focused handlers for system, singleton, capability, and verification events
use anchor_lang::prelude::*;
use super::{EventHandler, EventData, EventType};

/// Specialized event handler for system events
pub struct SystemEventHandler {
    pub handler_id: String,
}

impl EventHandler for SystemEventHandler {
    fn handle_event(&self, event: &EventData) -> Result<()> {
        match event.event_type {
            EventType::SystemInitialized => {
                msg!("System initialized successfully");
            }
            EventType::SystemPaused => {
                msg!("System paused for maintenance");
            }
            EventType::SystemResumed => {
                msg!("System resumed operations");
            }
            _ => {}
        }
        Ok(())
    }
    
    fn supported_event_types(&self) -> Vec<EventType> {
        vec![
            EventType::SystemInitialized,
            EventType::SystemPaused,
            EventType::SystemResumed,
        ]
    }
    
    fn priority(&self) -> u32 {
        1 // High priority for system events
    }
}

/// Specialized event handler for singleton events
pub struct SingletonEventHandler {
    pub handler_id: String,
    pub singleton_name: String,
}

impl EventHandler for SingletonEventHandler {
    fn handle_event(&self, event: &EventData) -> Result<()> {
        let singleton_name = event.metadata.iter()
            .find(|(key, _)| key == "singleton")
            .map(|(_, value)| value.as_str())
            .unwrap_or("unknown");
            
        if singleton_name == self.singleton_name {
            match event.event_type {
                EventType::ProcessorInitialized | 
                EventType::SchedulerInitialized | 
                EventType::DiffInitialized => {
                    msg!("Singleton {} initialized", singleton_name);
                }
                EventType::CapabilityProcessed => {
                    msg!("Capability processed by {}", singleton_name);
                }
                EventType::ExecutionScheduled => {
                    msg!("Execution scheduled by {}", singleton_name);
                }
                EventType::DiffCalculated | EventType::DiffProcessed => {
                    msg!("Diff operation completed by {}", singleton_name);
                }
                _ => {}
            }
        }
        Ok(())
    }
    
    fn supported_event_types(&self) -> Vec<EventType> {
        vec![
            EventType::ProcessorInitialized,
            EventType::ProcessorPaused,
            EventType::ProcessorResumed,
            EventType::CapabilityProcessed,
            EventType::SchedulerInitialized,
            EventType::ExecutionScheduled,
            EventType::QueueProcessed,
            EventType::ResourcesAllocated,
            EventType::PartialOrdersComposed,
            EventType::DiffInitialized,
            EventType::DiffCalculated,
            EventType::DiffProcessed,
            EventType::BatchOptimized,
        ]
    }
    
    fn priority(&self) -> u32 {
        50
    }
}

/// Specialized event handler for capability events
pub struct CapabilityEventHandler {
    pub handler_id: String,
}

impl EventHandler for CapabilityEventHandler {
    fn handle_event(&self, event: &EventData) -> Result<()> {
        let capability_id = event.metadata.iter()
            .find(|(key, _)| key == "capability_id")
            .map(|(_, value)| value.as_str())
            .unwrap_or("unknown");
            
        match event.event_type {
            EventType::CapabilityGranted => {
                msg!("Capability '{}' granted", capability_id);
            }
            EventType::CapabilityUpdated => {
                msg!("Capability '{}' updated", capability_id);
            }
            EventType::CapabilityRevoked => {
                msg!("Capability '{}' revoked", capability_id);
            }
            EventType::CapabilityExecuted => {
                msg!("Capability '{}' executed", capability_id);
            }
            _ => {}
        }
        Ok(())
    }
    
    fn supported_event_types(&self) -> Vec<EventType> {
        vec![
            EventType::CapabilityGranted,
            EventType::CapabilityUpdated,
            EventType::CapabilityRevoked,
            EventType::CapabilityExecuted,
        ]
    }
    
    fn priority(&self) -> u32 {
        75
    }
}

/// Specialized event handler for verification events
pub struct VerificationEventHandler {
    pub handler_id: String,
}

impl EventHandler for VerificationEventHandler {
    fn handle_event(&self, event: &EventData) -> Result<()> {
        let function_count = event.metadata.iter()
            .find(|(key, _)| key == "function_count")
            .and_then(|(_, value)| value.parse::<usize>().ok())
            .unwrap_or(0);
            
        match event.event_type {
            EventType::VerificationStarted => {
                msg!("Verification started with {} functions", function_count);
            }
            EventType::VerificationCompleted => {
                msg!("Verification completed successfully for {} functions", function_count);
            }
            EventType::VerificationFailed => {
                msg!("Verification failed for {} functions", function_count);
            }
            _ => {}
        }
        Ok(())
    }
    
    fn supported_event_types(&self) -> Vec<EventType> {
        vec![
            EventType::VerificationStarted,
            EventType::VerificationCompleted,
            EventType::VerificationFailed,
        ]
    }
    
    fn priority(&self) -> u32 {
        60
    }
}

/// Specialized event handler for session events
pub struct SessionEventHandler {
    pub handler_id: String,
}

impl EventHandler for SessionEventHandler {
    fn handle_event(&self, event: &EventData) -> Result<()> {
        let session_id = event.metadata.iter()
            .find(|(key, _)| key == "session_id")
            .map(|(_, value)| value.as_str())
            .unwrap_or("unknown");
            
        match event.event_type {
            EventType::SessionCreated => {
                msg!("Session '{}' created", session_id);
            }
            EventType::SessionActivated => {
                msg!("Session '{}' activated", session_id);
            }
            EventType::SessionClosed => {
                msg!("Session '{}' closed", session_id);
            }
            EventType::SessionDataUpdated => {
                msg!("Session '{}' data updated", session_id);
            }
            _ => {}
        }
        Ok(())
    }
    
    fn supported_event_types(&self) -> Vec<EventType> {
        vec![
            EventType::SessionCreated,
            EventType::SessionActivated,
            EventType::SessionClosed,
            EventType::SessionDataUpdated,
        ]
    }
    
    fn priority(&self) -> u32 {
        80
    }
}

/// Specialized event handler for function execution events
pub struct FunctionEventHandler {
    pub handler_id: String,
}

impl EventHandler for FunctionEventHandler {
    fn handle_event(&self, event: &EventData) -> Result<()> {
        let function_hash = event.metadata.iter()
            .find(|(key, _)| key == "function_hash")
            .map(|(_, value)| value.as_str())
            .unwrap_or("unknown");
            
        match event.event_type {
            EventType::FunctionExecuted => {
                msg!("Function '{}' executed", function_hash);
            }
            EventType::FunctionChainExecuted => {
                msg!("Function chain containing '{}' executed", function_hash);
            }
            EventType::FunctionAggregated => {
                msg!("Function '{}' results aggregated", function_hash);
            }
            _ => {}
        }
        Ok(())
    }
    
    fn supported_event_types(&self) -> Vec<EventType> {
        vec![
            EventType::FunctionExecuted,
            EventType::FunctionChainExecuted,
            EventType::FunctionAggregated,
        ]
    }
    
    fn priority(&self) -> u32 {
        90
    }
}

/// Specialized event handler for registry events
pub struct RegistryEventHandler {
    pub handler_id: String,
}

impl EventHandler for RegistryEventHandler {
    fn handle_event(&self, event: &EventData) -> Result<()> {
        let item_name = event.metadata.iter()
            .find(|(key, _)| key == "item_name")
            .map(|(_, value)| value.as_str())
            .unwrap_or("unknown");
            
        let item_version = event.metadata.iter()
            .find(|(key, _)| key == "item_version")
            .map(|(_, value)| value.as_str())
            .unwrap_or("unknown");
            
        match event.event_type {
            EventType::LibraryRegistered => {
                msg!("Library '{}' v{} registered", item_name, item_version);
            }
            EventType::ZkProgramRegistered => {
                msg!("ZK program '{}' v{} registered", item_name, item_version);
            }
            EventType::DependencyAdded => {
                msg!("Dependency '{}' v{} added", item_name, item_version);
            }
            _ => {}
        }
        Ok(())
    }
    
    fn supported_event_types(&self) -> Vec<EventType> {
        vec![
            EventType::LibraryRegistered,
            EventType::ZkProgramRegistered,
            EventType::DependencyAdded,
        ]
    }
    
    fn priority(&self) -> u32 {
        100
    }
} 