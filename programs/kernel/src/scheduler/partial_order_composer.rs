// Partial order composition for multi-shard coordination
// This module implements algorithms to compose partial orders from multiple shards

use anchor_lang::prelude::*;
use std::collections::{HashMap, HashSet};

/// Partial order constraint types that can be composed cleanly  
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub enum OrderingConstraint {
    /// capability_a must execute before capability_b
    Before { capability_a: String, capability_b: String },
    /// capability_a must execute after capability_b  
    After { capability_a: String, capability_b: String },
    /// capabilities can execute concurrently
    Concurrent { capabilities: Vec<String> },
    /// capabilities must execute in sequential order
    Sequential { capabilities: Vec<String> },
    /// capability has specific priority level (0-255)
    Priority { capability: String, level: u8 },
}

/// A partial order from a single shard
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ShardPartialOrder {
    /// Shard that owns this partial order
    pub shard_id: Pubkey,
    /// List of ordering constraints from this shard
    pub constraints: Vec<OrderingConstraint>,
    /// Version of the partial order (for updates)
    pub version: u64,
    /// Priority weight for this shard (higher = more important)
    pub priority_weight: u8,
}

/// Composed partial order from multiple shards
#[derive(Debug, Clone)]
pub struct ComposedPartialOrder {
    /// All capabilities mentioned across all shards
    pub capabilities: HashSet<String>,
    /// Composed constraints (conflicts resolved)
    pub constraints: Vec<OrderingConstraint>,
    /// Dependency graph for topological sorting
    pub dependency_graph: HashMap<String, HashSet<String>>,
    /// Priority assignments for capabilities
    pub priorities: HashMap<String, u8>,
    /// Shards that contributed to this composition
    pub contributing_shards: Vec<Pubkey>,
}

/// Result of composition attempt
#[derive(Debug, Clone)]
pub enum CompositionResult {
    /// Successful composition
    Success(ComposedPartialOrder),
    /// Composition failed due to conflicts
    Conflict(CompositionConflict),
    /// Composition failed due to cycles
    Cycle(Vec<String>),
}

/// Information about a composition conflict
#[derive(Debug, Clone)]
pub struct CompositionConflict {
    /// The conflicting constraints
    pub conflicting_constraints: Vec<(Pubkey, OrderingConstraint, Pubkey, OrderingConstraint)>,
    /// Capabilities involved in the conflict
    pub conflicting_capabilities: Vec<String>,
    /// Suggested resolution strategy
    pub resolution_strategy: ConflictResolutionStrategy,
}

/// Strategies for resolving composition conflicts
#[derive(Debug, Clone)]
pub enum ConflictResolutionStrategy {
    /// Use priority weights to resolve conflicts
    PriorityBased,
    /// Use first-comes-first-served resolution
    TemporalBased,
    /// Use majority consensus from shards
    ConsensusBased,
    /// Require manual intervention
    Manual,
}

/// Partial order composer implementation
pub struct PartialOrderComposer;

impl PartialOrderComposer {
    /// Compose multiple shard partial orders into a single composed order
    pub fn compose_partial_orders(
        shard_orders: Vec<ShardPartialOrder>,
    ) -> Result<CompositionResult> {
        if shard_orders.is_empty() {
            return Ok(CompositionResult::Success(ComposedPartialOrder {
                capabilities: HashSet::new(),
                constraints: vec![],
                dependency_graph: HashMap::new(),
                priorities: HashMap::new(),
                contributing_shards: vec![],
            }));
        }

        // Step 1: Collect all capabilities and constraints
        let mut all_capabilities = HashSet::new();
        let mut all_constraints = Vec::new();
        let mut shard_constraint_map: HashMap<Pubkey, Vec<OrderingConstraint>> = HashMap::new();
        let contributing_shards: Vec<Pubkey> = shard_orders.iter().map(|o| o.shard_id).collect();

        for shard_order in &shard_orders {
            for constraint in &shard_order.constraints {
                Self::extract_capabilities(constraint, &mut all_capabilities);
                all_constraints.push((shard_order.shard_id, constraint.clone()));
                shard_constraint_map
                    .entry(shard_order.shard_id)
                    .or_default()
                    .push(constraint.clone());
            }
        }

        // Step 2: Detect conflicts between shards
        if let Some(conflict) = Self::detect_conflicts(&shard_orders, &shard_constraint_map) {
            return Ok(CompositionResult::Conflict(conflict));
        }

        // Step 3: Build dependency graph
        let dependency_graph = Self::build_dependency_graph(&all_constraints)?;

        // Step 4: Check for cycles
        if let Some(cycle) = Self::detect_cycles(&dependency_graph) {
            return Ok(CompositionResult::Cycle(cycle));
        }

        // Step 5: Compute priorities
        let priorities = Self::compute_priorities(&shard_orders, &all_capabilities);

        // Step 6: Resolve remaining constraints
        let resolved_constraints = Self::resolve_constraints(all_constraints, &priorities);

        let composed_order = ComposedPartialOrder {
            capabilities: all_capabilities,
            constraints: resolved_constraints,
            dependency_graph,
            priorities,
            contributing_shards,
        };

        Ok(CompositionResult::Success(composed_order))
    }

    /// Extract capabilities mentioned in a constraint
    fn extract_capabilities(constraint: &OrderingConstraint, capabilities: &mut HashSet<String>) {
        match constraint {
            OrderingConstraint::Before { capability_a, capability_b } => {
                capabilities.insert(capability_a.clone());
                capabilities.insert(capability_b.clone());
            }
            OrderingConstraint::After { capability_a, capability_b } => {
                capabilities.insert(capability_a.clone());
                capabilities.insert(capability_b.clone());
            }
            OrderingConstraint::Concurrent { capabilities: caps } => {
                for cap in caps {
                    capabilities.insert(cap.clone());
                }
            }
            OrderingConstraint::Sequential { capabilities: caps } => {
                for cap in caps {
                    capabilities.insert(cap.clone());
                }
            }
            OrderingConstraint::Priority { capability, level: _ } => {
                capabilities.insert(capability.clone());
            }
        }
    }

    /// Detect conflicts between different shards' constraints
    fn detect_conflicts(
        shard_orders: &[ShardPartialOrder],
        shard_constraint_map: &HashMap<Pubkey, Vec<OrderingConstraint>>,
    ) -> Option<CompositionConflict> {
        let mut conflicts = Vec::new();

        // Check for direct ordering conflicts
        for (i, shard1) in shard_orders.iter().enumerate() {
            for shard2 in shard_orders.iter().skip(i + 1) {
                if let Some(constraints1) = shard_constraint_map.get(&shard1.shard_id) {
                    if let Some(constraints2) = shard_constraint_map.get(&shard2.shard_id) {
                        for c1 in constraints1 {
                            for c2 in constraints2 {
                                if Self::constraints_conflict(c1, c2) {
                                    conflicts.push((shard1.shard_id, c1.clone(), shard2.shard_id, c2.clone()));
                                }
                            }
                        }
                    }
                }
            }
        }

        if conflicts.is_empty() {
            None
        } else {
            let conflicting_capabilities = conflicts
                .iter()
                .flat_map(|(_, c1, _, c2)| {
                    let mut caps = HashSet::new();
                    Self::extract_capabilities(c1, &mut caps);
                    Self::extract_capabilities(c2, &mut caps);
                    caps
                })
                .collect();

            // Determine resolution strategy based on conflict type
            let resolution_strategy = if shard_orders.iter().any(|o| o.priority_weight > 0) {
                ConflictResolutionStrategy::PriorityBased
            } else {
                ConflictResolutionStrategy::ConsensusBased
            };

            Some(CompositionConflict {
                conflicting_constraints: conflicts,
                conflicting_capabilities,
                resolution_strategy,
            })
        }
    }

    /// Check if two constraints conflict with each other
    fn constraints_conflict(c1: &OrderingConstraint, c2: &OrderingConstraint) -> bool {
        match (c1, c2) {
            // A before B conflicts with B before A
            (
                OrderingConstraint::Before { capability_a: a1, capability_b: b1 },
                OrderingConstraint::Before { capability_a: a2, capability_b: b2 },
            ) => a1 == b2 && b1 == a2,
            
            // A before B conflicts with A after B
            (
                OrderingConstraint::Before { capability_a: a1, capability_b: b1 },
                OrderingConstraint::After { capability_a: a2, capability_b: b2 },
            ) => a1 == a2 && b1 == b2,
            
            // Sequential conflicts with concurrent for overlapping capabilities
            (
                OrderingConstraint::Sequential { capabilities: seq },
                OrderingConstraint::Concurrent { capabilities: conc },
            ) => seq.iter().any(|s| conc.contains(s)),
            
            // Add more conflict detection patterns as needed
            _ => false,
        }
    }

    /// Build dependency graph from constraints
    fn build_dependency_graph(
        constraints: &[(Pubkey, OrderingConstraint)],
    ) -> Result<HashMap<String, HashSet<String>>> {
        let mut graph: HashMap<String, HashSet<String>> = HashMap::new();

        for (_, constraint) in constraints {
            match constraint {
                OrderingConstraint::Before { capability_a, capability_b } => {
                    graph.entry(capability_b.clone())
                        .or_default()
                        .insert(capability_a.clone());
                }
                OrderingConstraint::After { capability_a, capability_b } => {
                    graph.entry(capability_a.clone())
                        .or_default()
                        .insert(capability_b.clone());
                }
                OrderingConstraint::Sequential { capabilities } => {
                    for i in 1..capabilities.len() {
                        graph.entry(capabilities[i].clone())
                            .or_default()
                            .insert(capabilities[i - 1].clone());
                    }
                }
                // Concurrent and Priority constraints don't add dependencies
                _ => {}
            }
        }

        Ok(graph)
    }

    /// Detect cycles in the dependency graph using DFS
    fn detect_cycles(graph: &HashMap<String, HashSet<String>>) -> Option<Vec<String>> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        for node in graph.keys() {
            if !visited.contains(node) {
                if let Some(cycle) = Self::dfs_cycle_detection(
                    node,
                    graph,
                    &mut visited,
                    &mut rec_stack,
                    &mut path,
                ) {
                    return Some(cycle);
                }
            }
        }

        None
    }

    /// DFS-based cycle detection
    fn dfs_cycle_detection(
        node: &str,
        graph: &HashMap<String, HashSet<String>>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
    ) -> Option<Vec<String>> {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());
        path.push(node.to_string());

        if let Some(neighbors) = graph.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    if let Some(cycle) = Self::dfs_cycle_detection(
                        neighbor,
                        graph,
                        visited,
                        rec_stack,
                        path,
                    ) {
                        return Some(cycle);
                    }
                } else if rec_stack.contains(neighbor) {
                    // Found a cycle
                    let cycle_start = path.iter().position(|x| x == neighbor).unwrap();
                    return Some(path[cycle_start..].to_vec());
                }
            }
        }

        rec_stack.remove(node);
        path.pop();
        None
    }

    /// Compute priority assignments for capabilities
    fn compute_priorities(
        shard_orders: &[ShardPartialOrder],
        capabilities: &HashSet<String>,
    ) -> HashMap<String, u8> {
        let mut priorities = HashMap::new();

        // Initialize all capabilities with default priority
        for cap in capabilities {
            priorities.insert(cap.clone(), 128); // Medium priority
        }

        // Apply priority constraints from shards (weighted by shard priority)
        for shard_order in shard_orders {
            for constraint in &shard_order.constraints {
                if let OrderingConstraint::Priority { capability, level } = constraint {
                    // Weight the priority by the shard's priority weight
                    let weighted_priority = ((*level as u16 * shard_order.priority_weight as u16) / 255) as u8;
                    let current_priority = priorities.get(capability).unwrap_or(&128);
                    
                    // Use the higher priority
                    if weighted_priority > *current_priority {
                        priorities.insert(capability.clone(), weighted_priority);
                    }
                }
            }
        }

        priorities
    }

    /// Resolve conflicts in constraints using priority-based resolution
    fn resolve_constraints(
        constraints: Vec<(Pubkey, OrderingConstraint)>,
        _priorities: &HashMap<String, u8>,
    ) -> Vec<OrderingConstraint> {
        // For now, return all constraints
        // More sophisticated resolution can be added later
        constraints.into_iter().map(|(_, constraint)| constraint).collect()
    }

    /// Check if a composed partial order is valid (no conflicts, no cycles)
    pub fn validate_composed_order(composed_order: &ComposedPartialOrder) -> Result<bool> {
        // Check for cycles in dependency graph
        if Self::detect_cycles(&composed_order.dependency_graph).is_some() {
            return Ok(false);
        }

        // Check for constraint consistency
        for constraint in &composed_order.constraints {
            match constraint {
                OrderingConstraint::Before { capability_a, capability_b } => {
                    if !composed_order.capabilities.contains(capability_a) ||
                       !composed_order.capabilities.contains(capability_b) {
                        return Ok(false);
                    }
                }
                OrderingConstraint::After { capability_a, capability_b } => {
                    if !composed_order.capabilities.contains(capability_a) ||
                       !composed_order.capabilities.contains(capability_b) {
                        return Ok(false);
                    }
                }
                OrderingConstraint::Sequential { capabilities } => {
                    for cap in capabilities {
                        if !composed_order.capabilities.contains(cap) {
                            return Ok(false);
                        }
                    }
                }
                OrderingConstraint::Concurrent { capabilities } => {
                    for cap in capabilities {
                        if !composed_order.capabilities.contains(cap) {
                            return Ok(false);
                        }
                    }
                }
                OrderingConstraint::Priority { capability, level: _ } => {
                    if !composed_order.capabilities.contains(capability) {
                        return Ok(false);
                    }
                }
            }
        }

        Ok(true)
    }
} 