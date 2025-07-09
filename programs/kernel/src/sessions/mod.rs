/// Session state management with lifecycle management, isolation, and factory patterns
/// This module provides session account structures, instruction handlers, isolation, and lifecycle
pub mod state;
pub mod instructions;
pub mod lifecycle;
pub mod isolation;

// Re-export specific types to avoid ambiguity
pub use state::{SessionState, SessionMetadata, SessionData, SessionConfiguration, SessionPermissions, SessionSettings as StateSessionSettings, SessionTemplate, SessionReservation};
pub use instructions::{
    initialize, execute_call, create_token_account, transfer_token, transfer_sol,
    approve_token, store_data, retrieve_data, update_metadata, close_session,
    Initialize, ExecuteCall, CreateTokenAccount, TransferToken, TransferSol,
    ApproveToken, StoreData, RetrieveData, UpdateMetadata, CloseSession,
};
pub use lifecycle::{SessionFactoryState, SessionEntry, SessionStatus, SessionState as LifecycleSessionState, SessionSettings as LifecycleSessionSettings, SessionCreationAttestation, CreationParameters};
pub use isolation::{SessionIsolationConfig, NamespaceCapability, ObjectId, Diff, MetadataEntry};
// Import error type from the unified error module
pub use crate::error::ValenceSessionError; 