// Function registry for valence-kernel registered operation resolution
//
// The valence-kernel supports calling registered functions through a hardcoded registry
// system that maps function IDs to program addresses. This provides a secure way to
// enable CPI calls to verified function implementations while maintaining the kernel's
// security guarantees through static verification.
//
// KERNEL INTEGRATION: Batch operations can reference registered functions by ID,
// and the kernel resolves these to actual program addresses using this registry.
// This enables dynamic function calls while maintaining security through a curated
// set of approved function implementations.
//
// SECURITY MODEL: The hardcoded registry ensures that only verified function
// implementations can be called through the kernel, preventing execution of
// arbitrary programs while enabling extensibility through registered functions.
use anchor_lang::prelude::*;

/// Information about a registered function
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct FunctionInfo {
    /// Program ID that implements this function
    pub program_id: Pubkey,
    /// Whether this function is currently active
    pub is_active: bool,
    /// Function name for debugging
    pub name: [u8; 32],
    /// Name length
    pub name_len: u8,
}

impl FunctionInfo {
    /// Get the hardcoded registry for minimal implementation
    pub fn get_registry_entry(registry_id: u64) -> Option<FunctionInfo> {
        match registry_id {
            // Example: ZK Verification Gateway
            1000 => Some(FunctionInfo {
                // Uses a deterministic PDA for the ZK gateway program
                program_id: Pubkey::find_program_address(
                    &[b"zk_gateway", &registry_id.to_le_bytes()],
                    &crate::ID
                ).0,
                is_active: true,
                name: *b"ZK Verification Gateway         ",
                name_len: 23,
            }),
            // Example: Token Swap Function  
            2000 => Some(FunctionInfo {
                // Uses a deterministic PDA for the swap program
                program_id: Pubkey::find_program_address(
                    &[b"token_swap", &registry_id.to_le_bytes()],
                    &crate::ID
                ).0,
                is_active: true,
                name: *b"Token Swap                      ",
                name_len: 10,
            }),
            _ => None,
        }
    }
}