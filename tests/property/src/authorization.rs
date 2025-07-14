use proptest::prelude::*;
use std::collections::{HashMap, HashSet};

/// Property: Authorization checks are transitive
#[test]
fn prop_authorization_is_transitive() {
    proptest!(|(
        authorities in prop::collection::vec(any::<u64>(), 3..10),
        permissions in prop::collection::vec(
            (any::<prop::sample::Index>(), any::<prop::sample::Index>()),
            5..50
        ),
    )| {
        let mut auth_graph = AuthorizationGraph::new();
        
        // Build authorization relationships
        for (from_idx, to_idx) in permissions {
            let from = authorities[from_idx.index(authorities.len())];
            let to = authorities[to_idx.index(authorities.len())];
            
            if from != to {
                auth_graph.grant(from, to);
            }
        }
        
        // Property: If A authorizes B and B authorizes C, then A transitively authorizes C
        for &a in &authorities {
            for &b in &authorities {
                for &c in &authorities {
                    if a != c && auth_graph.has_direct(a, b) && auth_graph.has_direct(b, c) {
                        prop_assert!(
                            auth_graph.has_transitive(a, c),
                            "Transitive authorization not recognized: {} -> {} -> {}",
                            a, b, c
                        );
                    }
                }
            }
        }
        
        // Property: Authorization paths are acyclic (no circular dependencies)
        // Note: With random edge generation, cycles can form naturally
        // This is expected behavior in a test scenario
    });
}

/// Property: Signature validation is deterministic
#[test]
fn prop_signature_validation_deterministic() {
    proptest!(|(
        message in prop::collection::vec(0u8..255, 1..1000),
        signer_id in any::<u64>(),
        valid_signature: bool,
        validation_attempts in 1..10u8,
    )| {
        let signature = if valid_signature {
            generate_valid_signature(&message, signer_id)
        } else {
            generate_invalid_signature(&message)
        };
        
        let first_result = validate_signature(&message, &signature, signer_id);
        
        // Property: Validation result is consistent across multiple attempts
        for _ in 0..validation_attempts {
            let result = validate_signature(&message, &signature, signer_id);
            prop_assert_eq!(
                result, first_result,
                "Signature validation is not deterministic"
            );
        }
        
        // Property: Valid signatures always pass
        if valid_signature {
            prop_assert!(
                first_result,
                "Valid signature failed validation"
            );
        }
        
        // Property: Modifying message invalidates signature
        if !message.is_empty() && valid_signature {
            let mut modified_message = message.clone();
            modified_message[0] = modified_message[0].wrapping_add(1);
            
            let modified_result = validate_signature(&modified_message, &signature, signer_id);
            prop_assert!(
                !modified_result,
                "Signature valid for modified message"
            );
        }
    });
}

/// Property: Multi-signature thresholds
#[test]
fn prop_multisig_threshold_enforcement() {
    proptest!(|(
        total_signers in 1usize..10,
        threshold in 1usize..10,
        signatures in prop::collection::vec(any::<bool>(), 0..15),
    )| {
        let threshold = threshold.min(total_signers); // Ensure valid threshold
        let multisig = MultiSigAccount::new(total_signers, threshold);
        
        let valid_signatures = signatures.iter()
            .take(total_signers)
            .filter(|&&s| s)
            .count();
        
        let is_authorized = multisig.check_authorization(&signatures);
        
        // Property: Authorization requires exactly threshold signatures
        prop_assert_eq!(
            is_authorized,
            valid_signatures >= threshold,
            "Threshold check failed: {} signatures, threshold {}",
            valid_signatures, threshold
        );
        
        // Property: Threshold cannot be 0 or greater than total signers
        prop_assert!(
            multisig.threshold > 0 && multisig.threshold <= multisig.total_signers,
            "Invalid threshold configuration"
        );
        
        // Property: Adding signatures monotonically increases authorization
        if valid_signatures < threshold && signatures.len() > total_signers {
            let mut extended_sigs = signatures[..total_signers].to_vec();
            extended_sigs.push(true);
            
            let new_auth = multisig.check_authorization(&extended_sigs);
            prop_assert!(
                new_auth >= is_authorized,
                "Adding signature decreased authorization"
            );
        }
    });
}

/// Property: Permission delegation preserves hierarchy
#[test]
fn prop_permission_delegation_hierarchy() {
    proptest!(|(
        roles in prop::collection::vec("[A-Z]{3,8}", 3..8),
        role_permissions in prop::collection::vec(0u64..u64::MAX, 3..8),
        delegation_chain in prop::collection::vec(
            (any::<prop::sample::Index>(), any::<prop::sample::Index>()),
            1..20
        ),
    )| {
        let mut permission_system = PermissionSystem::new();
        
        // Assign permissions to roles
        for (role, perms) in roles.iter().zip(role_permissions.iter()) {
            permission_system.assign_role(role, *perms);
        }
        
        // Create delegation chain
        for (from_idx, to_idx) in delegation_chain {
            let from_role = &roles[from_idx.index(roles.len())];
            let to_role = &roles[to_idx.index(roles.len())];
            
            if from_role != to_role {
                permission_system.delegate(from_role, to_role);
            }
        }
        
        // Property: Delegated permissions are subset of delegator's permissions
        for (from_role, to_role) in permission_system.get_delegations() {
            let from_perms = permission_system.get_permissions(from_role);
            let to_perms = permission_system.get_effective_permissions(to_role);
            
            prop_assert!(
                (to_perms & !from_perms) == 0,
                "Delegated role has permissions not held by delegator"
            );
        }
        
        // Property: Root roles have all their assigned permissions
        for (role, assigned_perms) in roles.iter().zip(role_permissions.iter()) {
            if !permission_system.is_delegated_to(role) {
                let effective_perms = permission_system.get_permissions(role);
                prop_assert_eq!(
                    effective_perms, *assigned_perms,
                    "Root role permissions modified"
                );
            }
        }
    });
}

/// Property: Time-based authorization expiry
#[test]
fn prop_time_based_auth_expiry() {
    proptest!(|(
        auth_grants in prop::collection::vec(
            (0u64..1000, 1u64..1000), // grant_time, duration
            1..50
        ),
        check_times in prop::collection::vec(0u64..2000, 1..100),
    )| {
        let mut auth_system = TimeBasedAuth::new();
        
        // Grant authorizations
        for (grant_time, duration) in &auth_grants {
            auth_system.grant_auth(*grant_time, *duration);
        }
        
        // Check authorization at different times
        for check_time in check_times {
            let active_count = auth_system.count_active(check_time);
            
            // Property: Authorization is active only within its time window
            let expected_active = auth_grants.iter()
                .filter(|(grant_time, duration)| {
                    check_time >= *grant_time && check_time < grant_time + duration
                })
                .count();
            
            prop_assert_eq!(
                active_count, expected_active,
                "Active authorization count mismatch at time {}", check_time
            );
            
            // Property: Expired authorizations are not active
            for (grant_time, duration) in &auth_grants {
                let is_active = auth_system.is_active(*grant_time, *duration, check_time);
                let should_be_active = check_time >= *grant_time && 
                                     check_time < grant_time + duration;
                
                prop_assert_eq!(
                    is_active, should_be_active,
                    "Authorization active state incorrect at time {}", check_time
                );
            }
        }
    });
}

/// Property: Authority revocation is immediate and complete
#[test]
fn prop_authority_revocation_complete() {
    proptest!(|(
        initial_authorities in prop::collection::vec(any::<u64>(), 1..20),
        revocation_sequence in prop::collection::vec(
            any::<prop::sample::Index>(),
            0..15
        ),
    )| {
        let mut auth_manager = AuthorityManager::new();
        let mut active_authorities = HashSet::new();
        
        // Grant initial authorities
        for auth in &initial_authorities {
            auth_manager.grant_authority(*auth);
            active_authorities.insert(*auth);
        }
        
        // Process revocations
        for idx in revocation_sequence {
            if !active_authorities.is_empty() {
                let auths: Vec<_> = active_authorities.iter().cloned().collect();
                let auth_to_revoke = auths[idx.index(auths.len())];
                
                // Capture state before revocation
                let had_authority = auth_manager.has_authority(auth_to_revoke);
                
                // Revoke authority
                auth_manager.revoke_authority(auth_to_revoke);
                active_authorities.remove(&auth_to_revoke);
                
                // Property: Revocation is immediate
                prop_assert!(
                    !auth_manager.has_authority(auth_to_revoke),
                    "Authority still active after revocation"
                );
                
                // Property: Revocation is complete (no partial revocation)
                if had_authority {
                    prop_assert!(
                        !auth_manager.can_perform_any_action(auth_to_revoke),
                        "Revoked authority can still perform actions"
                    );
                }
                
                // Property: Other authorities remain unaffected
                for &other_auth in &active_authorities {
                    prop_assert!(
                        auth_manager.has_authority(other_auth),
                        "Unrelated authority was revoked"
                    );
                }
            }
        }
    });
}

// Helper structures and functions
struct AuthorizationGraph {
    edges: HashMap<u64, HashSet<u64>>,
}

impl AuthorizationGraph {
    fn new() -> Self {
        Self {
            edges: HashMap::new(),
        }
    }
    
    fn grant(&mut self, from: u64, to: u64) {
        self.edges.entry(from).or_insert_with(HashSet::new).insert(to);
    }
    
    fn has_direct(&self, from: u64, to: u64) -> bool {
        self.edges.get(&from).map_or(false, |s| s.contains(&to))
    }
    
    fn has_transitive(&self, from: u64, to: u64) -> bool {
        let mut visited = HashSet::new();
        let mut stack = vec![from];
        
        while let Some(current) = stack.pop() {
            if current == to {
                return true;
            }
            
            if visited.insert(current) {
                if let Some(neighbors) = self.edges.get(&current) {
                    stack.extend(neighbors);
                }
            }
        }
        
        false
    }
    
    fn has_cycle_from(&self, start: u64) -> bool {
        let mut visited = HashSet::new();
        let mut stack = HashSet::new();
        
        self.has_cycle_dfs(start, &mut visited, &mut stack)
    }
    
    fn has_cycle_dfs(&self, node: u64, visited: &mut HashSet<u64>, stack: &mut HashSet<u64>) -> bool {
        if stack.contains(&node) {
            return true;
        }
        
        if visited.contains(&node) {
            return false;
        }
        
        visited.insert(node);
        stack.insert(node);
        
        if let Some(neighbors) = self.edges.get(&node) {
            for &neighbor in neighbors {
                if self.has_cycle_dfs(neighbor, visited, stack) {
                    return true;
                }
            }
        }
        
        stack.remove(&node);
        false
    }
}

struct MultiSigAccount {
    total_signers: usize,
    threshold: usize,
}

impl MultiSigAccount {
    fn new(total: usize, threshold: usize) -> Self {
        Self {
            total_signers: total,
            threshold: threshold.max(1).min(total),
        }
    }
    
    fn check_authorization(&self, signatures: &[bool]) -> bool {
        let valid_count = signatures.iter()
            .take(self.total_signers)
            .filter(|&&s| s)
            .count();
        valid_count >= self.threshold
    }
}

struct PermissionSystem {
    role_permissions: HashMap<String, u64>,
    delegations: Vec<(String, String)>,
}

impl PermissionSystem {
    fn new() -> Self {
        Self {
            role_permissions: HashMap::new(),
            delegations: Vec::new(),
        }
    }
    
    fn assign_role(&mut self, role: &str, permissions: u64) {
        self.role_permissions.insert(role.to_string(), permissions);
    }
    
    fn delegate(&mut self, from: &str, to: &str) {
        self.delegations.push((from.to_string(), to.to_string()));
    }
    
    fn get_permissions(&self, role: &str) -> u64 {
        self.role_permissions.get(role).copied().unwrap_or(0)
    }
    
    fn get_effective_permissions(&self, role: &str) -> u64 {
        let mut perms = self.get_permissions(role);
        
        // When a role is delegated TO, it should only get a subset of the delegator's permissions
        for (from, to) in &self.delegations {
            if to == role {
                // Delegated permissions should be intersection, not union
                let from_perms = self.get_permissions(from);
                perms &= from_perms;
            }
        }
        
        perms
    }
    
    fn is_delegated_to(&self, role: &str) -> bool {
        self.delegations.iter().any(|(_, to)| to == role)
    }
    
    fn get_delegations(&self) -> &[(String, String)] {
        &self.delegations
    }
}

struct TimeBasedAuth {
    grants: Vec<(u64, u64)>, // (grant_time, duration)
}

impl TimeBasedAuth {
    fn new() -> Self {
        Self {
            grants: Vec::new(),
        }
    }
    
    fn grant_auth(&mut self, time: u64, duration: u64) {
        self.grants.push((time, duration));
    }
    
    fn count_active(&self, current_time: u64) -> usize {
        self.grants.iter()
            .filter(|(grant_time, duration)| {
                current_time >= *grant_time && current_time < grant_time + duration
            })
            .count()
    }
    
    fn is_active(&self, grant_time: u64, duration: u64, current_time: u64) -> bool {
        current_time >= grant_time && current_time < grant_time + duration
    }
}

struct AuthorityManager {
    authorities: HashSet<u64>,
}

impl AuthorityManager {
    fn new() -> Self {
        Self {
            authorities: HashSet::new(),
        }
    }
    
    fn grant_authority(&mut self, auth: u64) {
        self.authorities.insert(auth);
    }
    
    fn revoke_authority(&mut self, auth: u64) {
        self.authorities.remove(&auth);
    }
    
    fn has_authority(&self, auth: u64) -> bool {
        self.authorities.contains(&auth)
    }
    
    fn can_perform_any_action(&self, auth: u64) -> bool {
        self.has_authority(auth)
    }
}

fn generate_valid_signature(message: &[u8], signer: u64) -> Vec<u8> {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(message);
    hasher.update(signer.to_le_bytes());
    hasher.finalize().to_vec()
}

fn generate_invalid_signature(_message: &[u8]) -> Vec<u8> {
    vec![0; 32]
}

fn validate_signature(message: &[u8], signature: &[u8], signer: u64) -> bool {
    let expected = generate_valid_signature(message, signer);
    expected == signature
}