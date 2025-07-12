//! Capability bitmap system for O(1) permission checks

/// Capability enum representing different permissions
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Capability {
    // Core capabilities (0-15)
    Read = 0,
    Write = 1,
    Execute = 2,
    Delete = 3,
    Admin = 4,
    Transfer = 5,
    Mint = 6,
    Burn = 7,
    CreateAccount = 8,
    CloseAccount = 9,
    Cpi = 10,
    Upgrade = 11,
    
    // Function capabilities (16-31)
    CallFunction = 16,
    RegisterFunction = 17,
    DeregisterFunction = 18,
    ImportFunction = 19,
    
    // State capabilities (32-47)
    UpdateState = 32,
    ReadState = 33,
    ValidateState = 34,
    MergeState = 35,
    
    // Session capabilities (48-63)
    ConsumeSession = 48,
    CreateSession = 49,
    UpdateSession = 50,
}

impl Capability {
    /// Convert capability to its bit position
    pub fn bit_position(&self) -> u8 {
        *self as u8
    }
    
    /// Convert capability to its bit mask
    pub fn to_mask(&self) -> u64 {
        1u64 << self.bit_position()
    }
    
    /// Parse capability from string (for backward compatibility during migration)
    pub fn from_string(s: &str) -> Option<Self> {
        match s {
            "read" => Some(Self::Read),
            "write" => Some(Self::Write),
            "execute" => Some(Self::Execute),
            "delete" => Some(Self::Delete),
            "admin" => Some(Self::Admin),
            "transfer" => Some(Self::Transfer),
            "mint" => Some(Self::Mint),
            "burn" => Some(Self::Burn),
            "create_account" => Some(Self::CreateAccount),
            "close_account" => Some(Self::CloseAccount),
            "cpi" => Some(Self::Cpi),
            "upgrade" => Some(Self::Upgrade),
            "call_function" => Some(Self::CallFunction),
            "register_function" => Some(Self::RegisterFunction),
            "deregister_function" => Some(Self::DeregisterFunction),
            "import_function" => Some(Self::ImportFunction),
            "update_state" => Some(Self::UpdateState),
            "read_state" => Some(Self::ReadState),
            "validate_state" => Some(Self::ValidateState),
            "merge_state" => Some(Self::MergeState),
            "consume_session" => Some(Self::ConsumeSession),
            "create_session" => Some(Self::CreateSession),
            "update_session" => Some(Self::UpdateSession),
            _ => None,
        }
    }
}

/// Capabilities struct wrapping a u64 bitmap
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Capabilities(pub u64);

impl Capabilities {
    /// Create empty capabilities
    pub fn none() -> Self {
        Self(0)
    }
    
    /// Create capabilities with all bits set
    pub fn all() -> Self {
        Self(u64::MAX)
    }
    
    /// Add a single capability
    pub fn add(&mut self, cap: Capability) {
        self.0 |= cap.to_mask();
    }
    
    /// Remove a single capability
    pub fn remove(&mut self, cap: Capability) {
        self.0 &= !cap.to_mask();
    }
    
    /// Check if has a single capability
    pub fn has(&self, cap: Capability) -> bool {
        (self.0 & cap.to_mask()) != 0
    }
    
    /// Check if has all of the given capabilities
    pub fn has_all(&self, caps: &[Capability]) -> bool {
        caps.iter().all(|cap| self.has(*cap))
    }
    
    /// Check if has any of the given capabilities
    pub fn has_any(&self, caps: &[Capability]) -> bool {
        caps.iter().any(|cap| self.has(*cap))
    }
    
    /// Merge capabilities from another set (OR operation)
    pub fn merge(&mut self, other: Capabilities) {
        self.0 |= other.0;
    }
    
    /// Intersect capabilities with another set (AND operation)
    pub fn intersect(&mut self, other: Capabilities) {
        self.0 &= other.0;
    }
    
    /// Create from a list of capabilities
    pub fn from_list(caps: &[Capability]) -> Self {
        let mut result = Self::none();
        for cap in caps {
            result.add(*cap);
        }
        result
    }
    
    /// Convert string capabilities to bitmap (for migration)
    pub fn from_strings(caps: &[String]) -> Self {
        let mut result = Self::none();
        for cap_str in caps {
            if let Some(cap) = Capability::from_string(cap_str) {
                result.add(cap);
            }
        }
        result
    }
}

/// Helper function to aggregate capabilities from multiple accounts
pub fn aggregate_capabilities(account_capabilities: &[Vec<String>]) -> Capabilities {
    let mut aggregated = Capabilities::none();
    
    for caps in account_capabilities {
        let account_caps = Capabilities::from_strings(caps);
        aggregated.merge(account_caps);
    }
    
    aggregated
}

// Backward compatibility constants
pub const TRANSFER: &str = "transfer";
pub const MINT: &str = "mint";
pub const BURN: &str = "burn";
pub const ADMIN: &str = "admin";
pub const READ: &str = "read";
pub const WRITE: &str = "write";
pub const CREATE_ACCOUNT: &str = "create_account";
pub const CLOSE_ACCOUNT: &str = "close_account";
pub const CPI: &str = "cpi";
pub const UPGRADE: &str = "upgrade";

/// Check if a capability is valid (backward compatibility)
pub fn is_valid_capability(capability: &str) -> bool {
    Capability::from_string(capability).is_some()
}

/// Normalize capability string (lowercase, trimmed)
pub fn normalize_capability(capability: &str) -> String {
    capability.trim().to_lowercase()
}