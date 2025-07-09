use anchor_lang::prelude::*;

/// Session metadata for session management
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct SessionMetadata {
    pub description: String,
    pub tags: Vec<String>,
    pub max_lifetime: i64, // 0 = unlimited
}

impl Default for SessionMetadata {
    fn default() -> Self {
        Self {
            description: String::new(),
            tags: Vec::new(),
            max_lifetime: 0,
        }
    }
}

/// Session data account for storing key-value pairs
#[account]
pub struct SessionData {
    /// Session this data belongs to
    pub session: Pubkey,
    /// Data key
    pub key: String,
    /// Data value
    pub value: Vec<u8>,
    /// Creation timestamp
    pub created_at: i64,
    /// Last updated timestamp
    pub last_updated: i64,
    /// PDA bump seed
    pub bump: u8,
}

impl SessionData {
    pub fn get_space(key_len: usize, max_value_len: usize) -> usize {
        8 + // discriminator
        32 + // session
        4 + key_len + // key string
        4 + max_value_len + // value vec
        8 + // created_at
        8 + // last_updated
        1 // bump
    }
}

/// Session configuration for templates and reservations
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct SessionConfiguration {
    /// Maximum session duration in seconds
    pub max_duration: u64,
    /// Session permissions
    pub permissions: SessionPermissions,
    /// Session-specific settings
    pub settings: SessionSettings,
}

impl Default for SessionConfiguration {
    fn default() -> Self {
        Self {
            max_duration: 86400, // 24 hours
            permissions: SessionPermissions::default(),
            settings: SessionSettings::default(),
        }
    }
}

/// Session permissions
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct SessionPermissions {
    /// Can execute functions
    pub can_execute_functions: bool,
    /// Can compose function chains
    pub can_compose_functions: bool,
    /// Can read from other sessions
    pub can_read_cross_session: bool,
    /// Can write to other sessions
    pub can_write_cross_session: bool,
}

impl Default for SessionPermissions {
    fn default() -> Self {
        Self {
            can_execute_functions: true,
            can_compose_functions: true,
            can_read_cross_session: false,
            can_write_cross_session: false,
        }
    }
}

/// Session-specific settings
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct SessionSettings {
    /// Auto-expire session after max_duration
    pub auto_expire: bool,
    /// Log all session activity
    pub enable_logging: bool,
    /// Performance monitoring
    pub enable_monitoring: bool,
}

impl Default for SessionSettings {
    fn default() -> Self {
        Self {
            auto_expire: true,
            enable_logging: false,
            enable_monitoring: false,
        }
    }
}

/// Template for creating sessions with predefined configurations
#[account]
pub struct SessionTemplate {
    /// Unique template ID
    pub template_id: String,
    /// Template name for display
    pub name: String,
    /// Default eval program for sessions created from this template
    pub default_eval_program: Pubkey,
    /// Default namespaces assigned to sessions
    pub default_namespaces: Vec<String>,
    /// Default capabilities granted to sessions
    pub default_capabilities: Vec<String>,
    /// Session configuration parameters
    pub session_config: SessionConfiguration,
    /// Whether this template is active
    pub is_active: bool,
    /// Template creation timestamp
    pub created_at: i64,
    /// Last updated timestamp
    pub updated_at: i64,
    /// Template creator
    pub created_by: Pubkey,
    /// PDA bump seed
    pub bump: u8,
}

impl SessionTemplate {
    pub fn get_space(
        template_id_len: usize,
        name_len: usize,
        namespaces: &[String],
        capabilities: &[String],
    ) -> usize {
        8 + // discriminator
        4 + template_id_len + // template_id string
        4 + name_len + // name string
        32 + // default_eval_program
        4 + namespaces.iter().map(|n| 4 + n.len()).sum::<usize>() + // default_namespaces vec
        4 + capabilities.iter().map(|c| 4 + c.len()).sum::<usize>() + // default_capabilities vec
        std::mem::size_of::<SessionConfiguration>() + // session_config
        1 + // is_active
        8 + // created_at
        8 + // updated_at
        32 + // created_by
        1 // bump
    }
}

/// Reservation for two-phase session creation
#[account]
pub struct SessionReservation {
    /// Unique reservation ID
    pub reservation_id: String,
    /// Reserved session ID
    pub session_id: String,
    /// User who made the reservation
    pub reserved_by: Pubkey,
    /// Intended session owner
    pub session_owner: Pubkey,
    /// Template to use for session creation (optional)
    pub template_id: Option<String>,
    /// Custom session configuration (if not using template)
    pub session_config: Option<SessionConfiguration>,
    /// Reservation expiry timestamp
    pub expires_at: i64,
    /// Reservation creation timestamp
    pub reserved_at: i64,
    /// Whether this reservation has been used
    pub is_used: bool,
    /// PDA bump seed
    pub bump: u8,
}

impl SessionReservation {
    pub fn get_space(
        reservation_id_len: usize,
        session_id_len: usize,
        template_id_len: Option<usize>,
    ) -> usize {
        8 + // discriminator
        4 + reservation_id_len + // reservation_id string
        4 + session_id_len + // session_id string
        32 + // reserved_by
        32 + // session_owner
        1 + template_id_len.map(|len| 4 + len).unwrap_or(0) + // template_id option
        1 + std::mem::size_of::<SessionConfiguration>() + // session_config option
        8 + // expires_at
        8 + // reserved_at
        1 + // is_used
        1 // bump
    }
    
    /// Check if this reservation has expired
    pub fn is_expired(&self, current_timestamp: i64) -> bool {
        current_timestamp > self.expires_at
    }
    
    /// Check if this reservation is valid for use
    pub fn is_valid(&self, current_timestamp: i64) -> bool {
        !self.is_used && !self.is_expired(current_timestamp)
    }
} 

/// Session state account for managing session lifecycle
#[account]
pub struct SessionState {
    /// Session owner
    pub owner: Pubkey,
    /// Eval program that can execute capabilities
    pub eval_program: Pubkey,
    /// Unique session identifier
    pub session_id: String,
    /// Namespaces this session can access
    pub namespaces: Vec<String>,
    /// Whether the session is active
    pub is_active: bool,
    /// Session metadata
    pub metadata: SessionMetadata,
    /// Total number of executions
    pub total_executions: u64,
    /// Session creation timestamp
    pub created_at: i64,
    /// Last activity timestamp
    pub last_activity: i64,
    /// PDA bump seed
    pub bump: u8,
}

impl SessionState {
    pub fn get_space(
        session_id_len: usize,
        namespaces: &[String],
        metadata: &SessionMetadata,
    ) -> usize {
        8 + // discriminator
        32 + // owner
        32 + // eval_program
        4 + session_id_len + // session_id string
        4 + namespaces.iter().map(|n| 4 + n.len()).sum::<usize>() + // namespaces vec
        1 + // is_active
        SessionMetadata::get_space(metadata) + // metadata
        8 + // total_executions
        8 + // created_at
        8 + // last_activity
        1 // bump
    }
}

impl SessionMetadata {
    pub fn get_space(&self) -> usize {
        4 + self.description.len() + // description string
        4 + self.tags.iter().map(|t| 4 + t.len()).sum::<usize>() + // tags vec
        8 // max_lifetime
    }
} 