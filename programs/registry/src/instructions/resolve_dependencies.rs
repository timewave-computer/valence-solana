use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::RegistryError;
use std::collections::{HashMap, HashSet, VecDeque};

/// Resolve library dependencies and create dependency graph
pub fn handler(
    ctx: Context<ResolveDependencies>,
    library_program_id: Pubkey,
) -> Result<Vec<Pubkey>> {
    let library_info = &ctx.accounts.library_info;
    let dependency_graph = &mut ctx.accounts.dependency_graph;
    
    // Perform topological sort to resolve dependencies
    let resolved_order = topological_sort(library_program_id, &library_info.dependencies)?;
    
    // Update dependency graph
    dependency_graph.root_library = library_program_id;
    dependency_graph.resolved_order = resolved_order.clone();
    dependency_graph.is_valid = true;
    dependency_graph.last_resolved = Clock::get()?.unix_timestamp;
    
    msg!("Resolved dependencies for library {}: {} dependencies", 
         library_program_id, 
         resolved_order.len());
    
    Ok(resolved_order)
}

/// Perform topological sort to detect cycles and resolve dependency order
pub fn topological_sort(
    root_library: Pubkey,
    dependencies: &[LibraryDependency],
) -> Result<Vec<Pubkey>> {
    let mut graph: HashMap<Pubkey, Vec<Pubkey>> = HashMap::new();
    let mut in_degree: HashMap<Pubkey, usize> = HashMap::new();
    let mut all_nodes: HashSet<Pubkey> = HashSet::new();
    
    // Add root library
    all_nodes.insert(root_library);
    in_degree.insert(root_library, 0);
    
    // Build graph from dependencies
    for dep in dependencies {
        all_nodes.insert(dep.program_id);
        graph.entry(dep.program_id).or_insert_with(Vec::new).push(root_library);
        *in_degree.entry(root_library).or_insert(0) += 1;
        in_degree.entry(dep.program_id).or_insert(0);
    }
    
    // Kahn's algorithm for topological sorting
    let mut queue: VecDeque<Pubkey> = VecDeque::new();
    let mut result: Vec<Pubkey> = Vec::new();
    
    // Find all nodes with no incoming edges
    for (&node, &degree) in &in_degree {
        if degree == 0 {
            queue.push_back(node);
        }
    }
    
    while let Some(node) = queue.pop_front() {
        result.push(node);
        
        // For each neighbor of the current node
        if let Some(neighbors) = graph.get(&node) {
            for &neighbor in neighbors {
                if let Some(degree) = in_degree.get_mut(&neighbor) {
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push_back(neighbor);
                    }
                }
            }
        }
    }
    
    // Check for cycles
    if result.len() != all_nodes.len() {
        return Err(RegistryError::CircularDependency.into());
    }
    
    // Validate dependency graph size
    if result.len() > 20 {
        return Err(RegistryError::DependencyGraphTooLarge.into());
    }
    
    Ok(result)
} 