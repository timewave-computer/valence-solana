use anchor_lang::prelude::*;
use valence_kernel::guards::{Guard, GuardOp, SerializedGuard};
use valence_kernel::CPIManifestEntry;
use crate::{Result, SdkError};

/// Builder for composing guards
pub struct GuardBuilder {
    guard: Option<Guard>,
}

impl GuardBuilder {
    /// Create a new guard builder
    pub fn new() -> Self {
        Self { guard: None }
    }

    /// Create a guard that allows anyone
    pub fn anyone() -> Guard {
        Guard::AlwaysTrue
    }

    /// Create a guard for a specific owner
    pub fn owner(_pubkey: Pubkey) -> Guard {
        Guard::OwnerOnly
    }

    /// Create a guard for multiple signers (using owner for now)
    pub fn signers(_pubkeys: Vec<Pubkey>) -> Guard {
        Guard::OwnerOnly
    }

    /// Create a time-based expiration guard
    pub fn expiration(expires_at: i64) -> Guard {
        Guard::Expiration { expires_at }
    }

    /// Create a usage limit guard
    pub fn usage_limit(max: u64) -> Guard {
        Guard::UsageLimit { max }
    }

    /// Create an external guard
    pub fn external(program: Pubkey, data: Vec<u8>) -> Guard {
        Guard::External { 
            program, 
            data, 
            required_accounts: Vec::new() 
        }
    }

    /// Combine guards with AND logic
    pub fn and(guard1: Guard, guard2: Guard) -> Guard {
        Guard::And(Box::new(guard1), Box::new(guard2))
    }

    /// Combine guards with OR logic
    pub fn or(guard1: Guard, guard2: Guard) -> Guard {
        Guard::Or(Box::new(guard1), Box::new(guard2))
    }

    /// Invert a guard
    pub fn not(guard: Guard) -> Guard {
        Guard::Not(Box::new(guard))
    }

    /// Set the current guard
    pub fn with(mut self, guard: Guard) -> Self {
        self.guard = Some(guard);
        self
    }

    /// Combine with AND
    pub fn and_with(mut self, other: Guard) -> Self {
        self.guard = Some(match self.guard {
            Some(current) => Self::and(current, other),
            None => other,
        });
        self
    }

    /// Combine with OR
    pub fn or_with(mut self, other: Guard) -> Self {
        self.guard = Some(match self.guard {
            Some(current) => Self::or(current, other),
            None => other,
        });
        self
    }

    /// Build the final guard
    pub fn build(self) -> Option<Guard> {
        self.guard
    }
}

impl Default for GuardBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper macro for composing guards
#[macro_export]
macro_rules! compose_guards {
    (and: $guard1:expr, $guard2:expr) => {
        $crate::GuardBuilder::and($guard1, $guard2)
    };
    (or: $guard1:expr, $guard2:expr) => {
        $crate::GuardBuilder::or($guard1, $guard2)
    };
    (not: $guard:expr) => {
        $crate::GuardBuilder::not($guard)
    };
    (and: $guard1:expr, $guard2:expr, $($rest:expr),+) => {
        compose_guards!(and: $crate::GuardBuilder::and($guard1, $guard2), $($rest),+)
    };
    (or: $guard1:expr, $guard2:expr, $($rest:expr),+) => {
        compose_guards!(or: $crate::GuardBuilder::or($guard1, $guard2), $($rest),+)
    };
}

// ================================
// Guard Compiler (Client-Side Only)
// ================================

/// Compile a Guard into a SerializedGuard for on-chain execution
/// 
/// This function converts the high-level recursive Guard enum into
/// a flattened sequence of opcodes that can be efficiently evaluated on-chain.
/// 
/// # Important
/// 
/// This compiler should ONLY be used client-side (in the SDK or tests).
/// Never use this in on-chain code due to:
/// - Unpredictable gas consumption
/// - Recursion depth risks
/// - DoS attack vectors
pub fn compile_guard(guard: &Guard) -> Result<SerializedGuard> {
    let mut compiler = GuardCompiler::new();
    compiler.compile_guard(guard)?;
    Ok(compiler.build())
}

struct GuardCompiler {
    opcodes: Vec<GuardOp>,
    cpi_manifest: Vec<CPIManifestEntry>,
}

impl GuardCompiler {
    fn new() -> Self {
        Self {
            opcodes: Vec::new(),
            cpi_manifest: Vec::new(),
        }
    }
    
    fn build(self) -> SerializedGuard {
        SerializedGuard {
            opcodes: self.opcodes,
            cpi_manifest: self.cpi_manifest,
        }
    }
    
    fn compile_guard(&mut self, guard: &Guard) -> Result<()> {
        match guard {
            // Simple guards compile to state assertion opcodes
            Guard::AlwaysTrue => {
                self.opcodes.push(GuardOp::Terminate);
            }
            Guard::AlwaysFalse => {
                self.opcodes.push(GuardOp::Abort);
            }
            Guard::OwnerOnly => {
                self.opcodes.push(GuardOp::CheckOwner);
            }
            Guard::Expiration { expires_at } => {
                self.opcodes.push(GuardOp::CheckExpiry { timestamp: *expires_at });
            }
            Guard::UsageLimit { max } => {
                self.opcodes.push(GuardOp::CheckUsageLimit { limit: *max });
            }
            
            // Permission guards not yet implemented in current APU
            Guard::Permission { .. } => {
                return Err(SdkError::UnsupportedGuardType);
            }
            Guard::PermissionAccount { .. } => {
                return Err(SdkError::UnsupportedGuardType);
            }
            
            // Composite guards use control flow
            Guard::And(a, b) => {
                // For AND: check A, if false jump to end, then check B
                self.compile_guard(a)?;
                let b_start = self.opcodes.len() + 1;
                self.opcodes.push(GuardOp::JumpIfFalse { offset: 0 }); // Placeholder
                
                self.compile_guard(b)?;
                self.opcodes.push(GuardOp::Terminate);
                
                // Fix up the jump offset
                let jump_distance = (self.opcodes.len() - b_start) as u8;
                self.opcodes[b_start - 1] = GuardOp::JumpIfFalse { offset: jump_distance };
            }
            Guard::Or(a, b) => {
                // For OR: check A, if true jump to success, else check B
                // Compile A
                self.compile_guard(a)?;
                
                // If A is true (result_flag=true), we need to jump to success
                // We need to invert the logic since we only have JumpIfFalse
                // So we check if result is true by jumping if NOT false
                let jump_to_b = self.opcodes.len();
                self.opcodes.push(GuardOp::JumpIfFalse { offset: 0 }); // Placeholder
                
                // A was true, so skip B and terminate with success
                self.opcodes.push(GuardOp::Terminate);
                
                // B evaluation starts here
                let b_start = self.opcodes.len();
                self.compile_guard(b)?;
                
                // Fix the jump offset
                self.opcodes[jump_to_b] = GuardOp::JumpIfFalse { 
                    offset: (b_start - jump_to_b - 1) as u8 
                };
            }
            Guard::Not(inner) => {
                // For NOT: check inner, then invert result
                self.compile_guard(inner)?;
                self.opcodes.push(GuardOp::JumpIfFalse { offset: 2 });
                self.opcodes.push(GuardOp::Abort);
                self.opcodes.push(GuardOp::Terminate);
            }
            
            // External guards become CPI invocations
            Guard::External { program, data, required_accounts } => {
                let manifest_index = self.cpi_manifest.len() as u8;
                let account_indices: Vec<u8> = (0..required_accounts.len() as u8).collect();
                
                self.cpi_manifest.push(CPIManifestEntry {
                    program_id: *program,
                    account_indices,
                    data: data.clone(),
                });
                
                self.opcodes.push(GuardOp::Invoke { manifest_index });
            }
        }
        
        Ok(())
    }
    
}