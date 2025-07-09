// Valence Protocol Scheduler - Multi-shard coordination and execution ordering
// This module provides scheduling algorithms and partial order composition
pub mod instructions;
pub mod state;
pub mod queue_manager;
pub mod priority_engine;
pub mod resource_allocator;
pub mod partial_order_composer;
pub mod topological_scheduler;
pub mod ordering_coordinator;
pub mod session_queue;

// Re-export instructions except conflicting types
pub use instructions::{
    initialize, schedule_execution, process_queue, allocate_resources, 
    compose_partial_orders, PartialOrderSpec,
    Initialize, ScheduleExecution, ProcessQueue, AllocateResources, ComposePartialOrders
};

// Re-export state
pub use state::*;

// Re-export queue management - be specific to avoid conflicts
pub use queue_manager::{QueueManager, QueueItem, QueueItemStatus, QueueStatus};
pub use session_queue::{
    SessionOperationQueue, PendingOperation, OperationType, QueueStats,
    QueueManager as SessionQueueManager
};

// Re-export other modules
pub use priority_engine::*;
pub use resource_allocator::*;
pub use partial_order_composer::*;
pub use topological_scheduler::*;

// Re-export ordering coordinator except conflicting types
pub use ordering_coordinator::{
    OrderingCoordinator, CoordinationResult, CoordinationPlan, CoordinationStrategy,
    OrderingConflict, ConflictType, ExecutionContext, ResourceMetrics, ConstraintCache
}; 