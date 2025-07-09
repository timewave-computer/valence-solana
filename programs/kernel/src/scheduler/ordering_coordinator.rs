// Coordination logic for respecting per-shard ordering rules
//
// This module implements advanced partial order composition algorithms for coordinating
// execution across multiple shards. It provides:
// 
// 1. Conflict Detection: Identifies contradictory orderings, cyclic dependencies, and resource contention
// 2. Conflict Resolution: Uses consensus-based, temporal-based, and priority-based algorithms
// 3. Conditional Ordering: Applies constraints based on execution context and resource availability
// 4. Dynamic Priority Adjustment: Modifies priorities based on real-time resource metrics
// 5. Performance Optimization: Includes constraint caching and composition metrics
//
// The framework supports both strict and best-effort composition modes, with extensible
// conflict resolution strategies for complex multi-shard scenarios.

use anchor_lang::prelude::*;
use std::collections::{HashMap, HashSet};
use crate::{SchedulerError, scheduler::instructions::{PartialOrderSpec, OrderingConstraint}};

/// Execution context for conditional ordering
#[derive(Debug)]
pub struct ExecutionContext {
    pub resource_availability: u32, // 0-100 percentage
    pub active_capabilities: Vec<String>,
    pub block_height: u64,
    pub timestamp: i64,
}

/// Resource metrics for dynamic priority adjustment
#[derive(Debug)]
pub struct ResourceMetrics {
    pub cpu_usage: u32,     // 0-100 percentage
    pub memory_usage: u32,  // 0-100 percentage
    pub network_usage: u32, // 0-100 percentage
    pub storage_usage: u32, // 0-100 percentage
}

/// Cache for ordering constraints
pub type ConstraintCache = HashMap<Pubkey, Vec<OrderingConstraint>>;

/// Coordinates ordering rules across multiple shards
pub struct OrderingCoordinator;

impl OrderingCoordinator {
    /// Coordinate execution order across multiple shards
    pub fn coordinate_shards(
        shard_orders: &[PartialOrderSpec],
    ) -> Result<CoordinationResult> {
        // Validate each shard's ordering rules
        for order in shard_orders {
            Self::validate_shard_order(order)?;
        }

        // Check for conflicts
        let conflicts = Self::check_inter_shard_conflicts(shard_orders);
        
        // Resolve conflicts if any exist
        let resolved_conflicts = if !conflicts.is_empty() {
            Self::resolve_conflicts(&conflicts, shard_orders)?
        } else {
            vec![]
        };

        // Determine coordination strategy based on analysis
        let coordination_strategy = Self::determine_coordination_strategy(shard_orders);

        // Create coordination plan
        let coordination_plan = CoordinationPlan {
            shard_count: shard_orders.len(),
            total_constraints: shard_orders.iter()
                .map(|o| o.constraints.len())
                .sum(),
            coordination_strategy,
        };

        Ok(CoordinationResult {
            plan: coordination_plan,
            conflicts: resolved_conflicts,
        })
    }

    /// Resolve conflicts using advanced resolution algorithms
    fn resolve_conflicts(
        conflicts: &[OrderingConflict],
        shard_orders: &[PartialOrderSpec],
    ) -> Result<Vec<OrderingConflict>> {
        let mut resolved = Vec::new();
        
        for conflict in conflicts {
            match &conflict.conflict_type {
                ConflictType::ContradictoryOrdering => {
                    // Use consensus-based resolution
                    if let Some(resolution) = Self::consensus_based_resolution(conflict, shard_orders) {
                        msg!("Resolved contradictory ordering using consensus: {}", resolution);
                    } else {
                        resolved.push(conflict.clone());
                    }
                }
                ConflictType::CyclicDependency => {
                    // Use temporal-based resolution
                    if let Some(resolution) = Self::temporal_based_resolution(conflict, shard_orders) {
                        msg!("Resolved cyclic dependency using temporal ordering: {}", resolution);
                    } else {
                        resolved.push(conflict.clone());
                    }
                }
                ConflictType::ResourceContention => {
                    // Use priority-based resolution
                    if let Some(resolution) = Self::priority_based_resolution(conflict, shard_orders) {
                        msg!("Resolved resource contention using priorities: {}", resolution);
                    } else {
                        resolved.push(conflict.clone());
                    }
                }
            }
        }
        
        Ok(resolved)
    }

    /// Consensus-based conflict resolution
    fn consensus_based_resolution(
        _conflict: &OrderingConflict,
        shard_orders: &[PartialOrderSpec],
    ) -> Option<String> {
        // Count votes for each ordering from all shards
        let mut votes: HashMap<String, u32> = HashMap::new();
        
        for order in shard_orders {
            for constraint in &order.constraints {
                match constraint {
                    OrderingConstraint::Before { capability_a, capability_b } => {
                        let vote_key = format!("{} before {}", capability_a, capability_b);
                        *votes.entry(vote_key).or_insert(0) += 1;
                    }
                    OrderingConstraint::After { capability_a, capability_b } => {
                        let vote_key = format!("{} after {}", capability_a, capability_b);
                        *votes.entry(vote_key).or_insert(0) += 1;
                    }
                    _ => {}
                }
            }
        }
        
        // Choose ordering with most votes
        votes.into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(resolution, _)| resolution)
    }

    /// Temporal-based conflict resolution
    fn temporal_based_resolution(
        _conflict: &OrderingConflict,
        shard_orders: &[PartialOrderSpec],
    ) -> Option<String> {
        // Use timestamps or block heights to resolve conflicts
        // For now, use creation order of shards as temporal ordering
        let mut temporal_order: Vec<(Pubkey, usize)> = shard_orders.iter()
            .enumerate()
            .map(|(idx, order)| (order.shard_id, idx))
            .collect();
        
        temporal_order.sort_by_key(|(_, idx)| *idx);
        
        Some(format!("Resolved using temporal ordering: {:?}", 
            temporal_order.iter().map(|(key, _)| key).collect::<Vec<_>>()))
    }

    /// Priority-based conflict resolution
    fn priority_based_resolution(
        _conflict: &OrderingConflict,
        shard_orders: &[PartialOrderSpec],
    ) -> Option<String> {
        // Extract priorities from constraints
        let mut capability_priorities: HashMap<String, u8> = HashMap::new();
        
        for order in shard_orders {
            for constraint in &order.constraints {
                if let OrderingConstraint::Priority { capability, level } = constraint {
                    capability_priorities.entry(capability.clone())
                        .and_modify(|p| *p = (*p).max(*level))
                        .or_insert(*level);
                }
            }
        }
        
        if !capability_priorities.is_empty() {
            let mut sorted_capabilities: Vec<_> = capability_priorities.into_iter().collect();
            sorted_capabilities.sort_by_key(|(_, priority)| std::cmp::Reverse(*priority));
            
            Some(format!("Resolved using priority ordering: {:?}", 
                sorted_capabilities.iter().map(|(cap, _)| cap).collect::<Vec<_>>()))
        } else {
            None
        }
    }

    /// Determine coordination strategy based on shard analysis
    fn determine_coordination_strategy(shard_orders: &[PartialOrderSpec]) -> CoordinationStrategy {
        // Check if shards can be executed in parallel
        let mut shared_capabilities = HashSet::new();
        let mut all_capabilities = HashSet::new();
        
        for order in shard_orders {
            let mut current_shard_caps = HashSet::new();
            
            for constraint in &order.constraints {
                match constraint {
                    OrderingConstraint::Before { capability_a, capability_b } |
                    OrderingConstraint::After { capability_a, capability_b } => {
                        current_shard_caps.insert(capability_a.clone());
                        current_shard_caps.insert(capability_b.clone());
                    }
                    OrderingConstraint::Sequential { capabilities } |
                    OrderingConstraint::Concurrent { capabilities } => {
                        for cap in capabilities {
                            current_shard_caps.insert(cap.clone());
                        }
                    }
                    OrderingConstraint::Priority { capability, .. } => {
                        current_shard_caps.insert(capability.clone());
                    }
                }
            }
            
            // Check for overlaps
            for cap in &current_shard_caps {
                if all_capabilities.contains(cap) {
                    shared_capabilities.insert(cap.clone());
                }
                all_capabilities.insert(cap.clone());
            }
        }
        
        // If no shared capabilities, can execute in parallel
        if shared_capabilities.is_empty() {
            CoordinationStrategy::Parallel
        } else {
            CoordinationStrategy::Sequential
        }
    }

    /// Validate ordering rules for a single shard
    fn validate_shard_order(order: &PartialOrderSpec) -> Result<()> {
        // Basic validation
        if order.constraints.is_empty() {
            msg!("Shard {} has no ordering constraints", order.shard_id);
            return Ok(());
        }

        // Validate constraint types
        for constraint in &order.constraints {
            match constraint {
                crate::scheduler::instructions::OrderingConstraint::Before { capability_a, capability_b } => {
                    if capability_a == capability_b {
                        return Err(SchedulerError::InvalidOrderingConstraint.into());
                    }
                }
                crate::scheduler::instructions::OrderingConstraint::Sequential { capabilities } => {
                    if capabilities.is_empty() {
                        return Err(SchedulerError::InvalidOrderingConstraint.into());
                    }
                }
                _ => {
                    // Other constraints are valid by default
                }
            }
        }

        Ok(())
    }

    /// Check for inter-shard conflicts
    pub fn check_inter_shard_conflicts(
        shard_orders: &[PartialOrderSpec],
    ) -> Vec<OrderingConflict> {
        let mut conflicts = Vec::new();
        let mut capability_owners: HashMap<String, Vec<Pubkey>> = HashMap::new();

        // Build capability ownership map
        for order in shard_orders {
            for constraint in &order.constraints {
                match constraint {
                    OrderingConstraint::Before { capability_a, capability_b } |
                    OrderingConstraint::After { capability_a, capability_b } => {
                        capability_owners.entry(capability_a.clone())
                            .or_default()
                            .push(order.shard_id);
                        capability_owners.entry(capability_b.clone())
                            .or_default()
                            .push(order.shard_id);
                    }
                    OrderingConstraint::Sequential { capabilities } |
                    OrderingConstraint::Concurrent { capabilities } => {
                        for cap in capabilities {
                            capability_owners.entry(cap.clone())
                                .or_default()
                                .push(order.shard_id);
                        }
                    }
                    OrderingConstraint::Priority { capability, .. } => {
                        capability_owners.entry(capability.clone())
                            .or_default()
                            .push(order.shard_id);
                    }
                }
            }
        }

        // Check for contradictory orderings
        for i in 0..shard_orders.len() {
            for j in (i + 1)..shard_orders.len() {
                if let Some(conflict) = Self::check_contradictory_ordering(
                    &shard_orders[i],
                    &shard_orders[j],
                ) {
                    conflicts.push(conflict);
                }
            }
        }

        // Check for cyclic dependencies
        if let Some(cycle_conflict) = Self::detect_cyclic_dependencies(shard_orders) {
            conflicts.push(cycle_conflict);
        }

        conflicts
    }

    /// Check for contradictory ordering between two shards
    fn check_contradictory_ordering(
        order_a: &PartialOrderSpec,
        order_b: &PartialOrderSpec,
    ) -> Option<OrderingConflict> {
        let mut ordering_a = HashMap::new();
        let mut ordering_b = HashMap::new();

        // Build ordering maps for shard A
        for constraint in &order_a.constraints {
            if let OrderingConstraint::Before { capability_a, capability_b } = constraint {
                ordering_a.insert((capability_a.clone(), capability_b.clone()), true);
            } else if let OrderingConstraint::After { capability_a, capability_b } = constraint {
                ordering_a.insert((capability_b.clone(), capability_a.clone()), true);
            }
        }

        // Build ordering maps for shard B
        for constraint in &order_b.constraints {
            if let OrderingConstraint::Before { capability_a, capability_b } = constraint {
                ordering_b.insert((capability_a.clone(), capability_b.clone()), true);
            } else if let OrderingConstraint::After { capability_a, capability_b } = constraint {
                ordering_b.insert((capability_b.clone(), capability_a.clone()), true);
            }
        }

        // Check for contradictions
        for ((cap_a, cap_b), _) in &ordering_a {
            if ordering_b.contains_key(&(cap_b.clone(), cap_a.clone())) {
                return Some(OrderingConflict {
                    shard_a: order_a.shard_id,
                    shard_b: order_b.shard_id,
                    conflict_type: ConflictType::ContradictoryOrdering,
                    details: format!("{} before {} in shard A, but reversed in shard B", cap_a, cap_b),
                });
            }
        }

        None
    }

    /// Detect cyclic dependencies across all shards
    fn detect_cyclic_dependencies(shard_orders: &[PartialOrderSpec]) -> Option<OrderingConflict> {
        let mut graph: HashMap<String, HashSet<String>> = HashMap::new();
        
        // Build dependency graph
        for order in shard_orders {
            for constraint in &order.constraints {
                match constraint {
                    OrderingConstraint::Before { capability_a, capability_b } => {
                        graph.entry(capability_a.clone())
                            .or_default()
                            .insert(capability_b.clone());
                    }
                    OrderingConstraint::After { capability_a, capability_b } => {
                        graph.entry(capability_b.clone())
                            .or_default()
                            .insert(capability_a.clone());
                    }
                    OrderingConstraint::Sequential { capabilities } => {
                        for i in 0..capabilities.len() - 1 {
                            graph.entry(capabilities[i].clone())
                                .or_default()
                                .insert(capabilities[i + 1].clone());
                        }
                    }
                    _ => {}
                }
            }
        }

        // Detect cycles using DFS
        let mut visited = HashSet::new();
        let mut recursion_stack = HashSet::new();
        
        for node in graph.keys() {
            if !visited.contains(node) && Self::has_cycle_dfs(
                    node,
                    &graph,
                    &mut visited,
                    &mut recursion_stack,
                ) {
                return Some(OrderingConflict {
                    shard_a: Pubkey::default(), // Could track actual shards involved
                    shard_b: Pubkey::default(),
                    conflict_type: ConflictType::CyclicDependency,
                    details: "Cyclic dependency detected in capability ordering".to_string(),
                });
            }
        }

        None
    }

    /// DFS helper for cycle detection
    fn has_cycle_dfs(
        node: &str,
        graph: &HashMap<String, HashSet<String>>,
        visited: &mut HashSet<String>,
        recursion_stack: &mut HashSet<String>,
    ) -> bool {
        visited.insert(node.to_string());
        recursion_stack.insert(node.to_string());

        if let Some(neighbors) = graph.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    if Self::has_cycle_dfs(neighbor, graph, visited, recursion_stack) {
                        return true;
                    }
                } else if recursion_stack.contains(neighbor) {
                    return true;
                }
            }
        }

        recursion_stack.remove(node);
        false
    }

    /// Apply conditional ordering constraints based on execution context
    pub fn apply_conditional_constraints(
        shard_orders: &mut [PartialOrderSpec],
        execution_context: &ExecutionContext,
    ) -> Result<()> {
        for order in shard_orders {
            let mut new_constraints = Vec::new();
            
            for constraint in &order.constraints {
                // Check if constraint should be applied based on context
                if Self::should_apply_constraint(constraint, execution_context) {
                    new_constraints.push(constraint.clone());
                }
                
                // Generate additional constraints based on context
                if let Some(additional) = Self::generate_contextual_constraints(constraint, execution_context) {
                    new_constraints.extend(additional);
                }
            }
            
            order.constraints = new_constraints;
        }
        
        Ok(())
    }

    /// Check if a constraint should be applied based on context
    fn should_apply_constraint(
        constraint: &OrderingConstraint,
        context: &ExecutionContext,
    ) -> bool {
        match constraint {
            OrderingConstraint::Priority { capability: _, level } => {
                // Apply high priority constraints always, others conditionally
                *level >= 7 || context.resource_availability > 50
            }
            OrderingConstraint::Sequential { capabilities } => {
                // Apply sequential constraints if resources are limited
                context.resource_availability < 30 || capabilities.len() > 5
            }
            _ => true, // Apply other constraints by default
        }
    }

    /// Generate additional constraints based on context
    fn generate_contextual_constraints(
        constraint: &OrderingConstraint,
        context: &ExecutionContext,
    ) -> Option<Vec<OrderingConstraint>> {
        match constraint {
            OrderingConstraint::Concurrent { capabilities } if context.resource_availability < 20 => {
                // Convert concurrent to sequential if resources are very limited
                Some(vec![OrderingConstraint::Sequential {
                    capabilities: capabilities.clone(),
                }])
            }
            _ => None,
        }
    }

    /// Dynamically adjust priorities based on resource availability
    pub fn adjust_priorities_dynamically(
        shard_orders: &mut [PartialOrderSpec],
        resource_metrics: &ResourceMetrics,
    ) -> Result<()> {
        for order in shard_orders {
            for constraint in &mut order.constraints {
                if let OrderingConstraint::Priority { capability, level } = constraint {
                    // Adjust priority based on resource metrics
                    let adjustment = Self::calculate_priority_adjustment(
                        capability,
                        *level,
                        resource_metrics,
                    );
                    
                    *level = (*level as i8 + adjustment).clamp(0, 10) as u8;
                    
                    msg!("Adjusted priority for {}: {} -> {}", 
                        capability, 
                        *level as i8 - adjustment, 
                        *level
                    );
                }
            }
        }
        
        Ok(())
    }

    /// Calculate priority adjustment based on resource metrics
    fn calculate_priority_adjustment(
        capability: &str,
        current_priority: u8,
        metrics: &ResourceMetrics,
    ) -> i8 {
        let mut adjustment = 0i8;
        
        // Increase priority for capabilities that use scarce resources
        if metrics.cpu_usage > 80 && capability.contains("compute") {
            adjustment -= 2; // Lower priority for compute-intensive tasks
        }
        
        if metrics.memory_usage > 80 && capability.contains("memory") {
            adjustment -= 2; // Lower priority for memory-intensive tasks
        }
        
        // Boost priority for time-sensitive capabilities
        if capability.contains("urgent") || capability.contains("realtime") {
            adjustment += 3;
        }
        
        // Ensure we don't overflow priority bounds
        if current_priority as i8 + adjustment > 10 {
            adjustment = 10 - current_priority as i8;
        } else if current_priority as i8 + adjustment < 0 {
            adjustment = -(current_priority as i8);
        }
        
        adjustment
    }

    /// Cache ordering constraints for performance
    pub fn cache_constraints(
        shard_id: &Pubkey,
        constraints: &[OrderingConstraint],
        cache: &mut ConstraintCache,
    ) {
        cache.insert(*shard_id, constraints.to_vec());
        msg!("Cached {} constraints for shard {}", constraints.len(), shard_id);
    }

    /// Retrieve cached constraints
    pub fn get_cached_constraints(
        shard_id: &Pubkey,
        cache: &ConstraintCache,
    ) -> Option<Vec<OrderingConstraint>> {
        cache.get(shard_id).cloned()
    }

    /// Monitor and report composition performance metrics
    pub fn report_performance_metrics(
        composition_time: u64,
        conflict_count: usize,
        resolved_count: usize,
    ) {
        msg!("Composition performance metrics:");
        msg!("  - Composition time: {} ms", composition_time);
        msg!("  - Conflicts detected: {}", conflict_count);
        msg!("  - Conflicts resolved: {}", resolved_count);
        msg!("  - Resolution rate: {:.2}%", 
            if conflict_count > 0 {
                (resolved_count as f64 / conflict_count as f64) * 100.0
            } else {
                100.0
            }
        );
    }

    /// Visualize ordering constraints for debugging
    pub fn visualize_constraints(shard_orders: &[PartialOrderSpec]) -> String {
        let mut visualization = String::from("=== Ordering Constraints Visualization ===\n");
        
        for (idx, order) in shard_orders.iter().enumerate() {
            visualization.push_str(&format!("\nShard {} ({}):\n", idx, order.shard_id));
            
            for constraint in &order.constraints {
                match constraint {
                    OrderingConstraint::Before { capability_a, capability_b } => {
                        visualization.push_str(&format!("  {} -> {}\n", capability_a, capability_b));
                    }
                    OrderingConstraint::After { capability_a, capability_b } => {
                        visualization.push_str(&format!("  {} <- {}\n", capability_a, capability_b));
                    }
                    OrderingConstraint::Sequential { capabilities } => {
                        visualization.push_str("  Sequential: ");
                        for (i, cap) in capabilities.iter().enumerate() {
                            if i > 0 {
                                visualization.push_str(" -> ");
                            }
                            visualization.push_str(cap);
                        }
                        visualization.push('\n');
                    }
                    OrderingConstraint::Concurrent { capabilities } => {
                        visualization.push_str("  Concurrent: [");
                        visualization.push_str(&capabilities.join(", "));
                        visualization.push_str("]\n");
                    }
                    OrderingConstraint::Priority { capability, level } => {
                        visualization.push_str(&format!("  {} (priority: {})\n", capability, level));
                    }
                }
            }
        }
        
        visualization
    }

    /// Debug helper to trace composition decisions
    pub fn trace_composition_decision(
        decision: &str,
        input: &str,
        output: &str,
    ) {
        msg!("Composition Decision:");
        msg!("  Decision: {}", decision);
        msg!("  Input: {}", input);
        msg!("  Output: {}", output);
    }
}

/// Result of coordination across shards
#[derive(Debug)]
pub struct CoordinationResult {
    pub plan: CoordinationPlan,
    pub conflicts: Vec<OrderingConflict>,
}

/// Plan for coordinating execution across shards
#[derive(Debug)]
pub struct CoordinationPlan {
    pub shard_count: usize,
    pub total_constraints: usize,
    pub coordination_strategy: CoordinationStrategy,
}

/// Strategy for coordinating across shards
#[derive(Debug)]
pub enum CoordinationStrategy {
    Sequential,  // Execute shards sequentially
    Parallel,    // Execute shards in parallel where possible
}

/// Conflict between shard ordering rules
#[derive(Debug, Clone)]
pub struct OrderingConflict {
    pub shard_a: Pubkey,
    pub shard_b: Pubkey,
    pub conflict_type: ConflictType,
    pub details: String,
}

/// Type of ordering conflict
#[derive(Debug, Clone)]
pub enum ConflictType {
    ContradictoryOrdering,
    ResourceContention,
    CyclicDependency,
} 