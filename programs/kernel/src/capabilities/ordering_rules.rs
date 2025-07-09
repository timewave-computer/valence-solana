// App-specific ordering rules for shard contracts
// This module implements the partial order framework for capability execution
// COMPOSITION ALGORITHM APPROACH:
// The framework uses a multi-stage composition process:
// 1. Local Validation: Each shard validates its own constraints for consistency
// 2. Global Composition: The scheduler singleton composes partial orders from all shards
// 3. Conflict Detection: Identifies contradictory orderings, cycles, and resource contention
// 4. Conflict Resolution: Uses configurable strategies (consensus, temporal, priority-based)
// 5. Execution Planning: Generates final execution order respecting all constraints
// CURRENT LIMITATIONS:
// - Maximum 256 constraints per shard (can be increased with account resizing)
// - Conflict resolution is deterministic but may not be optimal for all use cases
// - No support for probabilistic or fuzzy constraints (planned for future)
// - Limited to static constraints (dynamic constraints require re-composition)
//
// TODO: Future enhancements
// - ConflictResolution::BestEffort mode for partial satisfaction of constraints
// - Dynamic constraint generation based on runtime conditions
// - Probabilistic ordering for non-deterministic workflows
// - Machine learning-based optimization of execution plans

use anchor_lang::prelude::*;
use crate::error::ValenceError;
use std::collections::{HashMap, HashSet};

/// Ordering constraint types that can be composed cleanly
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

/// Partial order specification for a shard
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct PartialOrder {
    /// Shard ID that owns this partial order
    pub shard_id: Pubkey,
    /// List of ordering constraints
    pub constraints: Vec<OrderingConstraint>,
    /// Version of the partial order (for updates)
    pub version: u64,
    /// Whether this partial order is active
    pub is_active: bool,
    /// Priority weight for this shard's constraints (0-255)
    pub priority_weight: u8,
    /// Metadata about this partial order
    pub metadata: PartialOrderMetadata,
}

/// Metadata for partial orders
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Default)]
pub struct PartialOrderMetadata {
    /// Human-readable description
    pub description: String,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Creation timestamp
    pub created_at: i64,
    /// Last update timestamp
    pub updated_at: i64,
}

/// Validation result for ordering constraints
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether the constraint is valid
    pub is_valid: bool,
    /// Error messages if invalid
    pub errors: Vec<String>,
    /// Warnings (non-fatal issues)
    pub warnings: Vec<String>,
    /// Capabilities affected by this constraint
    pub affected_capabilities: HashSet<String>,
}

/// Enhanced partial order implementation
impl PartialOrder {
    /// Create a new partial order
    pub fn new(shard_id: Pubkey) -> Self {
        Self {
            shard_id,
            constraints: vec![],
            version: 1,
            is_active: true,
            priority_weight: 128, // Medium priority
            metadata: PartialOrderMetadata {
                description: String::new(),
                tags: vec![],
                created_at: Clock::get().map(|c| c.unix_timestamp).unwrap_or(0),
                updated_at: Clock::get().map(|c| c.unix_timestamp).unwrap_or(0),
            },
        }
    }
    
    /// Add an ordering constraint with validation
    pub fn add_constraint(mut self, constraint: OrderingConstraint) -> Result<Self> {
        let validation = Self::validate_constraint(&constraint, &self.constraints)?;
        
        if !validation.is_valid {
            return Err(ValenceError::ConflictingConstraints.into());
        }
        
        self.constraints.push(constraint);
        self.metadata.updated_at = Clock::get().map(|c| c.unix_timestamp).unwrap_or(0);
        Ok(self)
    }
    
    /// Validate a single constraint against existing constraints
    pub fn validate_constraint(
        new_constraint: &OrderingConstraint,
        existing_constraints: &[OrderingConstraint],
    ) -> Result<ValidationResult> {
        let mut result = ValidationResult {
            is_valid: true,
            errors: vec![],
            warnings: vec![],
            affected_capabilities: HashSet::new(),
        };

        // Basic constraint validation
        match new_constraint {
            OrderingConstraint::Before { capability_a, capability_b } => {
                if capability_a.is_empty() || capability_b.is_empty() {
                    result.is_valid = false;
                    result.errors.push("Capability names cannot be empty".to_string());
                }
                if capability_a == capability_b {
                    result.is_valid = false;
                    result.errors.push("Capability cannot have ordering constraint with itself".to_string());
                }
                result.affected_capabilities.insert(capability_a.clone());
                result.affected_capabilities.insert(capability_b.clone());
            }
            OrderingConstraint::After { capability_a, capability_b } => {
                if capability_a.is_empty() || capability_b.is_empty() {
                    result.is_valid = false;
                    result.errors.push("Capability names cannot be empty".to_string());
                }
                if capability_a == capability_b {
                    result.is_valid = false;
                    result.errors.push("Capability cannot have ordering constraint with itself".to_string());
                }
                result.affected_capabilities.insert(capability_a.clone());
                result.affected_capabilities.insert(capability_b.clone());
            }
            OrderingConstraint::Concurrent { capabilities } => {
                if capabilities.is_empty() {
                    result.is_valid = false;
                    result.errors.push("Concurrent constraint must have at least one capability".to_string());
                }
                if capabilities.len() == 1 {
                    result.warnings.push("Concurrent constraint with single capability is redundant".to_string());
                }
                let unique_caps: HashSet<_> = capabilities.iter().collect();
                if unique_caps.len() != capabilities.len() {
                    result.is_valid = false;
                    result.errors.push("Duplicate capabilities in concurrent constraint".to_string());
                }
                for cap in capabilities {
                    result.affected_capabilities.insert(cap.clone());
                }
            }
            OrderingConstraint::Sequential { capabilities } => {
                if capabilities.len() < 2 {
                    result.is_valid = false;
                    result.errors.push("Sequential constraint must have at least two capabilities".to_string());
                }
                let unique_caps: HashSet<_> = capabilities.iter().collect();
                if unique_caps.len() != capabilities.len() {
                    result.is_valid = false;
                    result.errors.push("Duplicate capabilities in sequential constraint".to_string());
                }
                for cap in capabilities {
                    result.affected_capabilities.insert(cap.clone());
                }
            }
            OrderingConstraint::Priority { capability, level } => {
                if capability.is_empty() {
                    result.is_valid = false;
                    result.errors.push("Capability name cannot be empty".to_string());
                }
                if *level == 0 {
                    result.warnings.push("Priority level 0 might indicate unused capability".to_string());
                }
                result.affected_capabilities.insert(capability.clone());
            }
        }

        // Check for conflicts with existing constraints
        for existing in existing_constraints {
            if Self::constraints_conflict(new_constraint, existing) {
                result.is_valid = false;
                result.errors.push(format!(
                    "New constraint conflicts with existing constraint: {existing:?}"
                ));
            }
        }

        Ok(result)
    }
    
    /// Check if this partial order conflicts with another
    pub fn has_conflict_with(&self, other: &PartialOrder) -> bool {
        for constraint1 in &self.constraints {
            for constraint2 in &other.constraints {
                if Self::constraints_conflict(constraint1, constraint2) {
                    return true;
                }
            }
        }
        false
    }
    
    /// Enhanced conflict detection between constraints
    fn constraints_conflict(c1: &OrderingConstraint, c2: &OrderingConstraint) -> bool {
        match (c1, c2) {
            // Direct ordering conflicts
            (
                OrderingConstraint::Before { capability_a: a1, capability_b: b1 },
                OrderingConstraint::Before { capability_a: a2, capability_b: b2 },
            ) => a1 == b2 && b1 == a2, // A before B conflicts with B before A
            
            (
                OrderingConstraint::Before { capability_a: a1, capability_b: b1 },
                OrderingConstraint::After { capability_a: a2, capability_b: b2 },
            ) => a1 == a2 && b1 == b2, // A before B conflicts with A after B
            
            // Reverse case: After before Before
            (
                OrderingConstraint::After { capability_a: a1, capability_b: b1 },
                OrderingConstraint::Before { capability_a: a2, capability_b: b2 },
            ) => a1 == a2 && b1 == b2, // A after B conflicts with A before B
            
            // Sequential vs Concurrent conflicts (both directions)
            (
                OrderingConstraint::Sequential { capabilities: seq },
                OrderingConstraint::Concurrent { capabilities: conc },
            ) => {
                // Sequential capabilities cannot be concurrent
                seq.iter().any(|s| conc.contains(s)) && seq.len() > 1
            }
            
            (
                OrderingConstraint::Concurrent { capabilities: conc },
                OrderingConstraint::Sequential { capabilities: seq },
            ) => {
                // Concurrent capabilities cannot be sequential
                seq.iter().any(|s| conc.contains(s)) && seq.len() > 1
            }
            
            // No conflicts for other combinations
            _ => false,
        }
    }
    
    /// Get all capabilities mentioned in this partial order
    pub fn get_capabilities(&self) -> Vec<String> {
        let mut capabilities = HashSet::new();
        
        for constraint in &self.constraints {
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
        
        capabilities.into_iter().collect()
    }
    
    /// Update the partial order version
    pub fn increment_version(&mut self) {
        self.version += 1;
        self.metadata.updated_at = Clock::get().map(|c| c.unix_timestamp).unwrap_or(0);
    }
    
    /// Validate the entire partial order for consistency
    pub fn validate_consistency(&self) -> Result<()> {
        // Check for self-conflicts
        for (i, constraint1) in self.constraints.iter().enumerate() {
            for constraint2 in self.constraints.iter().skip(i + 1) {
                require!(
                    !Self::constraints_conflict(constraint1, constraint2),
                    ValenceError::ConflictingConstraints
                );
            }
        }
        
        // Check for cycles
        let dependency_graph = self.build_dependency_graph()?;
        if self.has_cycles(&dependency_graph) {
            return Err(ValenceError::ConflictingConstraints.into());
        }
        
        Ok(())
    }

    /// Build dependency graph from constraints
    fn build_dependency_graph(&self) -> Result<HashMap<String, HashSet<String>>> {
        let mut graph: HashMap<String, HashSet<String>> = HashMap::new();

        for constraint in &self.constraints {
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
                // Concurrent and Priority don't add dependencies
                _ => {}
            }
        }

        Ok(graph)
    }

    /// Check for cycles using DFS
    fn has_cycles(&self, graph: &HashMap<String, HashSet<String>>) -> bool {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for node in graph.keys() {
            if !visited.contains(node) && self.dfs_has_cycle(node, graph, &mut visited, &mut rec_stack) {
                return true;
            }
        }
        false
    }

    /// DFS cycle detection helper
    fn dfs_has_cycle(
        &self,
        node: &str,
        graph: &HashMap<String, HashSet<String>>,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
    ) -> bool {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());

        if let Some(neighbors) = graph.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    if self.dfs_has_cycle(neighbor, graph, visited, rec_stack) {
                        return true;
                    }
                } else if rec_stack.contains(neighbor) {
                    return true; // Back edge found - cycle detected
                }
            }
        }

        rec_stack.remove(node);
        false
    }

    /// Set priority weight for this shard's constraints
    #[must_use]
    pub fn with_priority_weight(mut self, weight: u8) -> Self {
        self.priority_weight = weight;
        self
    }

    /// Add metadata to the partial order
    #[must_use]
    pub fn with_metadata(mut self, description: String, tags: Vec<String>) -> Self {
        self.metadata.description = description;
        self.metadata.tags = tags;
        self.metadata.updated_at = Clock::get().map(|c| c.unix_timestamp).unwrap_or(0);
        self
    }

    /// Check if this partial order is compatible with a set of capabilities
    pub fn is_compatible_with_capabilities(&self, available_capabilities: &[String]) -> bool {
        let required_capabilities: HashSet<String> = self.get_capabilities().into_iter().collect();
        let available_set: HashSet<String> = available_capabilities.iter().cloned().collect();
        
        required_capabilities.is_subset(&available_set)
    }

    /// Get statistics about this partial order
    pub fn get_statistics(&self) -> PartialOrderStatistics {
        let capabilities = self.get_capabilities();
        let mut constraint_type_counts = HashMap::new();

        for constraint in &self.constraints {
            let constraint_type = match constraint {
                OrderingConstraint::Before { .. } => "before",
                OrderingConstraint::After { .. } => "after",
                OrderingConstraint::Concurrent { .. } => "concurrent",
                OrderingConstraint::Sequential { .. } => "sequential",
                OrderingConstraint::Priority { .. } => "priority",
            };
            *constraint_type_counts.entry(constraint_type.to_string()).or_insert(0) += 1;
        }

        PartialOrderStatistics {
            total_capabilities: capabilities.len(),
            total_constraints: self.constraints.len(),
            constraint_type_counts,
            is_consistent: self.validate_consistency().is_ok(),
            priority_weight: self.priority_weight,
        }
    }
}

/// Statistics about a partial order
#[derive(Debug, Clone)]
pub struct PartialOrderStatistics {
    pub total_capabilities: usize,
    pub total_constraints: usize,
    pub constraint_type_counts: HashMap<String, usize>,
    pub is_consistent: bool,
    pub priority_weight: u8,
}

/// Ordering rule trait for implementing custom ordering logic
pub trait OrderingRule {
    /// Check if the rule applies to the given capabilities
    fn applies_to(&self, capabilities: &[String]) -> bool;
    
    /// Generate ordering constraints for the given capabilities
    fn generate_constraints(&self, capabilities: &[String]) -> Result<Vec<OrderingConstraint>>;
    
    /// Get the rule priority (higher = more important)
    fn get_priority(&self) -> u8;
}

/// FIFO (First-In-First-Out) ordering rule
#[derive(Debug, Clone)]
pub struct FifoOrderingRule;

impl OrderingRule for FifoOrderingRule {
    fn applies_to(&self, _capabilities: &[String]) -> bool {
        true // FIFO applies to all capabilities
    }
    
    fn generate_constraints(&self, capabilities: &[String]) -> Result<Vec<OrderingConstraint>> {
        if capabilities.len() <= 1 {
            return Ok(vec![]);
        }
        
        // Generate sequential ordering constraints
        Ok(vec![OrderingConstraint::Sequential {
            capabilities: capabilities.to_vec(),
        }])
    }
    
    fn get_priority(&self) -> u8 {
        10 // Low priority
    }
}

/// Priority-based ordering rule
#[derive(Debug, Clone)]
pub struct PriorityOrderingRule {
    pub priority_map: std::collections::HashMap<String, u8>,
}

impl PriorityOrderingRule {
    pub fn new(priority_map: std::collections::HashMap<String, u8>) -> Self {
        Self { priority_map }
    }
}

impl OrderingRule for PriorityOrderingRule {
    fn applies_to(&self, capabilities: &[String]) -> bool {
        capabilities.iter().any(|cap| self.priority_map.contains_key(cap))
    }
    
    fn generate_constraints(&self, capabilities: &[String]) -> Result<Vec<OrderingConstraint>> {
        let mut constraints = vec![];
        
        for capability in capabilities {
            if let Some(&priority) = self.priority_map.get(capability) {
                constraints.push(OrderingConstraint::Priority {
                    capability: capability.clone(),
                    level: priority,
                });
            }
        }
        
        Ok(constraints)
    }
    
    fn get_priority(&self) -> u8 {
        50 // Medium priority
    }
}

/// Dependency-based ordering rule
#[derive(Debug, Clone)]
pub struct DependencyOrderingRule {
    pub dependencies: std::collections::HashMap<String, Vec<String>>,
}

impl DependencyOrderingRule {
    pub fn new(dependencies: std::collections::HashMap<String, Vec<String>>) -> Self {
        Self { dependencies }
    }
}

impl OrderingRule for DependencyOrderingRule {
    fn applies_to(&self, capabilities: &[String]) -> bool {
        capabilities.iter().any(|cap| self.dependencies.contains_key(cap))
    }
    
    fn generate_constraints(&self, capabilities: &[String]) -> Result<Vec<OrderingConstraint>> {
        let mut constraints = vec![];
        
        for capability in capabilities {
            if let Some(deps) = self.dependencies.get(capability) {
                for dep in deps {
                    if capabilities.contains(dep) {
                        constraints.push(OrderingConstraint::Before {
                            capability_a: dep.clone(),
                            capability_b: capability.clone(),
                        });
                    }
                }
            }
        }
        
        Ok(constraints)
    }
    
    fn get_priority(&self) -> u8 {
        100 // High priority
    }
}

/// Registry for ordering rules
pub struct OrderingRuleRegistry {
    rules: Vec<Box<dyn OrderingRule>>,
}

impl OrderingRuleRegistry {
    pub fn new() -> Self {
        Self { rules: vec![] }
    }
    
    /// Add a rule to the registry
    pub fn add_rule(&mut self, rule: Box<dyn OrderingRule>) {
        self.rules.push(rule);
    }
    
    /// Generate all applicable constraints for given capabilities
    pub fn generate_all_constraints(&self, capabilities: &[String]) -> Result<Vec<OrderingConstraint>> {
        let mut all_constraints = vec![];
        
        // Sort rules by priority (highest first)
        let mut sorted_rules: Vec<_> = self.rules.iter().collect();
        sorted_rules.sort_by(|a, b| b.get_priority().cmp(&a.get_priority()));
        
        for rule in sorted_rules {
            if rule.applies_to(capabilities) {
                let constraints = rule.generate_constraints(capabilities)?;
                all_constraints.extend(constraints);
            }
        }
        
        Ok(all_constraints)
    }
}

impl Default for OrderingRuleRegistry {
    fn default() -> Self {
        let mut registry = Self::new();
        
        // Add default rules
        registry.add_rule(Box::new(FifoOrderingRule));
        
        registry
    }
} 