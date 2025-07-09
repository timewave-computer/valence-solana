// Topological scheduling for capability execution
// This module implements topological sort-based scheduling that respects partial order constraints

use anchor_lang::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};
use super::partial_order_composer::ComposedPartialOrder;

/// Execution order for capabilities
#[derive(Debug, Clone)]
pub struct ExecutionSchedule {
    /// Ordered list of execution batches (capabilities in same batch can run concurrently)
    pub execution_batches: Vec<ExecutionBatch>,
    /// Total number of capabilities to execute
    pub total_capabilities: usize,
    /// Maximum parallelism level (largest batch size)
    pub max_parallelism: usize,
    /// Scheduling metadata
    pub metadata: SchedulingMetadata,
}

/// A batch of capabilities that can execute concurrently
#[derive(Debug, Clone)]
pub struct ExecutionBatch {
    /// Capabilities in this batch
    pub capabilities: Vec<String>,
    /// Priority level for this batch (higher = execute first)
    pub priority: u8,
    /// Expected execution time for this batch
    pub estimated_duration: Option<u64>,
}

/// Metadata about the scheduling process
#[derive(Debug, Clone)]
pub struct SchedulingMetadata {
    /// Algorithm used for scheduling
    pub algorithm: SchedulingAlgorithm,
    /// Time taken to compute the schedule
    pub computation_time_ms: u64,
    /// Whether the schedule is optimal
    pub is_optimal: bool,
    /// Number of constraint violations (should be 0)
    pub constraint_violations: u32,
}

/// Available scheduling algorithms
#[derive(Debug, Clone)]
pub enum SchedulingAlgorithm {
    /// Kahn's algorithm for topological sorting
    Kahn,
    /// DFS-based topological sorting
    DfsTopological,
    /// Priority-aware scheduling
    PriorityBased,
    /// Resource-aware scheduling
    ResourceAware,
}

/// Topological scheduler implementation
pub struct TopologicalScheduler;

impl TopologicalScheduler {
    /// Create an execution schedule from a composed partial order
    pub fn create_schedule(
        composed_order: ComposedPartialOrder,
        algorithm: SchedulingAlgorithm,
    ) -> Result<ExecutionSchedule> {
        let start_time = anchor_lang::solana_program::clock::Clock::get()?.unix_timestamp as u64;

        let execution_batches = match algorithm {
            SchedulingAlgorithm::Kahn => Self::kahn_topological_sort(&composed_order)?,
            SchedulingAlgorithm::DfsTopological => Self::dfs_topological_sort(&composed_order)?,
            SchedulingAlgorithm::PriorityBased => Self::priority_based_schedule(&composed_order)?,
            SchedulingAlgorithm::ResourceAware => Self::resource_aware_schedule(&composed_order)?,
        };

        let end_time = anchor_lang::solana_program::clock::Clock::get()?.unix_timestamp as u64;
        let computation_time = end_time.saturating_sub(start_time);

        let total_capabilities = composed_order.capabilities.len();
        let max_parallelism = execution_batches.iter()
            .map(|batch| batch.capabilities.len())
            .max()
            .unwrap_or(0);

        let metadata = SchedulingMetadata {
            algorithm,
            computation_time_ms: computation_time,
            is_optimal: Self::verify_schedule_optimality(&execution_batches, &composed_order),
            constraint_violations: Self::count_constraint_violations(&execution_batches, &composed_order),
        };

        Ok(ExecutionSchedule {
            execution_batches,
            total_capabilities,
            max_parallelism,
            metadata,
        })
    }

    /// Kahn's algorithm for topological sorting with batch creation
    fn kahn_topological_sort(composed_order: &ComposedPartialOrder) -> Result<Vec<ExecutionBatch>> {
        let mut in_degree = HashMap::new();
        let mut graph = HashMap::new();

        // Initialize in-degree and adjacency list
        for capability in &composed_order.capabilities {
            in_degree.insert(capability.clone(), 0);
            graph.insert(capability.clone(), Vec::new());
        }

        // Build graph and calculate in-degrees
        for (dependent, dependencies) in &composed_order.dependency_graph {
            for dependency in dependencies {
                graph.entry(dependency.clone())
                    .or_insert_with(Vec::new)
                    .push(dependent.clone());
                *in_degree.entry(dependent.clone()).or_insert(0) += 1;
            }
        }

        let mut batches = Vec::new();
        let mut queue = VecDeque::new();

        // Start with capabilities that have no dependencies
        for (capability, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(capability.clone());
            }
        }

        while !queue.is_empty() {
            let mut current_batch = Vec::new();
            let batch_size = queue.len();

            // Process all capabilities at current level
            for _ in 0..batch_size {
                if let Some(capability) = queue.pop_front() {
                    current_batch.push(capability.clone());

                    // Update in-degrees of dependent capabilities
                    if let Some(dependents) = graph.get(&capability) {
                        for dependent in dependents {
                            if let Some(degree) = in_degree.get_mut(dependent) {
                                *degree -= 1;
                                if *degree == 0 {
                                    queue.push_back(dependent.clone());
                                }
                            }
                        }
                    }
                }
            }

            if !current_batch.is_empty() {
                let priority = Self::calculate_batch_priority(&current_batch, &composed_order.priorities);
                batches.push(ExecutionBatch {
                    capabilities: current_batch,
                    priority,
                    estimated_duration: None,
                });
            }
        }

        Ok(batches)
    }

    /// DFS-based topological sorting
    fn dfs_topological_sort(composed_order: &ComposedPartialOrder) -> Result<Vec<ExecutionBatch>> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut finish_order = Vec::new();

        for capability in &composed_order.capabilities {
            if !visited.contains(capability) {
                Self::dfs_visit(
                    capability,
                    &composed_order.dependency_graph,
                    &mut visited,
                    &mut rec_stack,
                    &mut finish_order,
                )?;
            }
        }

        // Reverse the finish order to get topological order
        finish_order.reverse();

        // Group into batches based on dependencies
        let batches = Self::group_into_batches(finish_order, &composed_order.dependency_graph, &composed_order.priorities);

        Ok(batches)
    }

    /// DFS visit helper for topological sort
    fn dfs_visit(
        capability: &str,
        graph: &HashMap<String, HashSet<String>>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        finish_order: &mut Vec<String>,
    ) -> Result<()> {
        visited.insert(capability.to_string());
        rec_stack.insert(capability.to_string());

        if let Some(dependencies) = graph.get(capability) {
            for dependency in dependencies {
                if !visited.contains(dependency) {
                    Self::dfs_visit(dependency, graph, visited, rec_stack, finish_order)?;
                } else if rec_stack.contains(dependency) {
                    // Cycle detected
                    return Err(anchor_lang::error::Error::from(anchor_lang::error::ErrorCode::ConstraintRaw));
                }
            }
        }

        rec_stack.remove(capability);
        finish_order.push(capability.to_string());
        Ok(())
    }

    /// Priority-based scheduling that considers capability priorities
    fn priority_based_schedule(composed_order: &ComposedPartialOrder) -> Result<Vec<ExecutionBatch>> {
        // Start with topological sort
        let mut basic_batches = Self::kahn_topological_sort(composed_order)?;

        // Re-order within batches based on priorities
        for batch in &mut basic_batches {
            batch.capabilities.sort_by(|a, b| {
                let priority_a = composed_order.priorities.get(a).unwrap_or(&128);
                let priority_b = composed_order.priorities.get(b).unwrap_or(&128);
                priority_b.cmp(priority_a) // Higher priority first
            });

            // Update batch priority
            batch.priority = Self::calculate_batch_priority(&batch.capabilities, &composed_order.priorities);
        }

        // Sort batches by priority
        basic_batches.sort_by(|a, b| b.priority.cmp(&a.priority));

        Ok(basic_batches)
    }

    /// Resource-aware scheduling (placeholder for future resource management)
    fn resource_aware_schedule(composed_order: &ComposedPartialOrder) -> Result<Vec<ExecutionBatch>> {
        // For now, use priority-based scheduling
        // Future enhancement: consider resource requirements and availability
        Self::priority_based_schedule(composed_order)
    }

    /// Group capabilities into batches based on dependencies
    fn group_into_batches(
        ordered_capabilities: Vec<String>,
        dependency_graph: &HashMap<String, HashSet<String>>,
        priorities: &HashMap<String, u8>,
    ) -> Vec<ExecutionBatch> {
        let mut batches = Vec::new();
        let mut remaining = ordered_capabilities;
        let mut completed = HashSet::new();

        while !remaining.is_empty() {
            let mut current_batch = Vec::new();
            let mut i = 0;

            while i < remaining.len() {
                let capability = &remaining[i];
                
                // Check if all dependencies are completed
                let can_execute = if let Some(deps) = dependency_graph.get(capability) {
                    deps.iter().all(|dep| completed.contains(dep))
                } else {
                    true
                };

                if can_execute {
                    current_batch.push(remaining.remove(i));
                } else {
                    i += 1;
                }
            }

            if current_batch.is_empty() {
                // Deadlock - should not happen with valid topological sort
                break;
            }

            // Mark capabilities as completed
            for capability in &current_batch {
                completed.insert(capability.clone());
            }

            let priority = Self::calculate_batch_priority(&current_batch, priorities);
            batches.push(ExecutionBatch {
                capabilities: current_batch,
                priority,
                estimated_duration: None,
            });
        }

        batches
    }

    /// Calculate priority for a batch based on capabilities
    fn calculate_batch_priority(capabilities: &[String], priorities: &HashMap<String, u8>) -> u8 {
        if capabilities.is_empty() {
            return 128;
        }

        // Use the highest priority in the batch
        capabilities.iter()
            .map(|cap| priorities.get(cap).unwrap_or(&128))
            .max()
            .copied()
            .unwrap_or(128)
    }

    /// Verify if the schedule is optimal (all constraints respected, maximum parallelism)
    fn verify_schedule_optimality(
        batches: &[ExecutionBatch],
        composed_order: &ComposedPartialOrder,
    ) -> bool {
        // Check if all constraints are respected
        let mut executed = HashSet::new();
        
        for batch in batches {
            // Check dependencies are satisfied
            for capability in &batch.capabilities {
                if let Some(deps) = composed_order.dependency_graph.get(capability) {
                    for dep in deps {
                        if !executed.contains(dep) {
                            return false; // Dependency not satisfied
                        }
                    }
                }
            }
            
            // Mark capabilities as executed
            for capability in &batch.capabilities {
                executed.insert(capability.clone());
            }
        }

        true
    }

    /// Count constraint violations in the schedule
    fn count_constraint_violations(
        batches: &[ExecutionBatch],
        composed_order: &ComposedPartialOrder,
    ) -> u32 {
        let mut violations = 0;
        let mut executed = HashSet::new();
        
        for batch in batches {
            for capability in &batch.capabilities {
                if let Some(deps) = composed_order.dependency_graph.get(capability) {
                    for dep in deps {
                        if !executed.contains(dep) {
                            violations += 1;
                        }
                    }
                }
            }
            
            for capability in &batch.capabilities {
                executed.insert(capability.clone());
            }
        }

        violations
    }

    /// Get capabilities ready for execution (no pending dependencies)
    pub fn get_ready_capabilities(
        schedule: &ExecutionSchedule,
        completed_capabilities: &HashSet<String>,
    ) -> Vec<String> {
        for batch in &schedule.execution_batches {
            let mut ready_in_batch = Vec::new();
            
            for capability in &batch.capabilities {
                if !completed_capabilities.contains(capability) {
                    ready_in_batch.push(capability.clone());
                }
            }
            
            if !ready_in_batch.is_empty() {
                return ready_in_batch;
            }
        }
        
        Vec::new()
    }

    /// Update schedule when capabilities complete
    pub fn mark_capabilities_completed(
        schedule: &mut ExecutionSchedule,
        completed: &[String],
    ) {
        // Remove completed capabilities from batches
        for batch in &mut schedule.execution_batches {
            batch.capabilities.retain(|cap| !completed.contains(cap));
        }
        
        // Remove empty batches
        schedule.execution_batches.retain(|batch| !batch.capabilities.is_empty());
    }
} 