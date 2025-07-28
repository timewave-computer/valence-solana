use crate::error::Result;
use anchor_lang::prelude::*;
use serde::{Deserialize, Serialize};

// ================================
// IDL Generation and Management
// ================================

/// Valence IDL format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValenceIdl {
    /// Version of the IDL format
    pub version: String,

    /// Program name
    pub name: String,

    /// Program ID
    pub program_id: String,

    /// Instructions
    pub instructions: Vec<IdlInstruction>,

    /// Accounts
    pub accounts: Vec<IdlAccount>,

    /// Types
    pub types: Vec<IdlType>,

    /// Errors
    pub errors: Vec<IdlError>,

    /// Metadata
    pub metadata: IdlMetadata,
}

/// IDL instruction definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlInstruction {
    /// Instruction name
    pub name: String,

    /// Instruction arguments
    pub args: Vec<IdlArg>,

    /// Required accounts
    pub accounts: Vec<IdlInstructionAccount>,

    /// Description
    pub description: Option<String>,
}

/// IDL account definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlAccount {
    /// Account name
    pub name: String,

    /// Account type
    pub account_type: IdlAccountType,

    /// Fields
    pub fields: Vec<IdlField>,
}

/// IDL type definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlType {
    /// Type name
    pub name: String,

    /// Type definition
    pub type_def: IdlTypeDef,
}

/// IDL error definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlError {
    /// Error code
    pub code: u32,

    /// Error name
    pub name: String,

    /// Error message
    pub msg: String,
}

/// IDL metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlMetadata {
    /// Protocol used
    pub protocol: String,

    /// Functions used
    pub functions: Vec<String>,

    /// Guards used
    pub guards: Vec<String>,

    /// Build info
    pub build_info: BuildInfo,
}

/// Build information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildInfo {
    /// Rust version
    pub rust_version: String,

    /// Anchor version
    pub anchor_version: String,

    /// Solana version
    pub solana_version: String,

    /// Build timestamp
    pub build_timestamp: i64,
}

// ================================
// IDL Type Definitions
// ================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlArg {
    pub name: String,
    pub arg_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlInstructionAccount {
    pub name: String,
    pub is_mut: bool,
    pub is_signer: bool,
    pub is_optional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlField {
    pub name: String,
    pub field_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum IdlAccountType {
    Account,
    ProgramAccount,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum IdlTypeDef {
    Struct { fields: Vec<IdlField> },
    Enum { variants: Vec<IdlEnumVariant> },
    Alias { value: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdlEnumVariant {
    pub name: String,
    pub fields: Option<Vec<IdlField>>,
}

// ================================
// IDL Generator
// ================================

pub struct IdlGenerator;

impl IdlGenerator {
    /// Generate IDL from program metadata
    pub fn generate(
        program_name: &str,
        program_id: &Pubkey,
        functions: Vec<String>,
        guards: Vec<String>,
    ) -> Result<ValenceIdl> {
        let idl = ValenceIdl {
            version: "0.1.0".to_string(),
            name: program_name.to_string(),
            program_id: program_id.to_string(),
            instructions: Self::generate_instructions(),
            accounts: Self::generate_accounts(),
            types: Self::generate_types(),
            errors: Self::generate_errors(),
            metadata: IdlMetadata {
                protocol: "valence".to_string(),
                functions,
                guards,
                build_info: BuildInfo {
                    rust_version: "1.75.0".to_string(),
                    anchor_version: "0.31.1".to_string(),
                    solana_version: "2.1.6".to_string(),
                    build_timestamp: chrono::Utc::now().timestamp(),
                },
            },
        };

        Ok(idl)
    }

    fn generate_instructions() -> Vec<IdlInstruction> {
        vec![IdlInstruction {
            name: "create_session_account".to_string(),
            args: vec![
                IdlArg {
                    name: "guard_hash".to_string(),
                    arg_type: "[u8; 32]".to_string(),
                },
                IdlArg {
                    name: "expires_at".to_string(),
                    arg_type: "i64".to_string(),
                },
            ],
            accounts: vec![
                IdlInstructionAccount {
                    name: "owner".to_string(),
                    is_mut: true,
                    is_signer: true,
                    is_optional: false,
                },
                IdlInstructionAccount {
                    name: "session".to_string(),
                    is_mut: true,
                    is_signer: false,
                    is_optional: false,
                },
            ],
            description: Some("Create a new session".to_string()),
        }]
    }

    fn generate_accounts() -> Vec<IdlAccount> {
        vec![IdlAccount {
            name: "Session".to_string(),
            account_type: IdlAccountType::Account,
            fields: vec![
                IdlField {
                    name: "owner".to_string(),
                    field_type: "Pubkey".to_string(),
                },
                IdlField {
                    name: "guard_hash".to_string(),
                    field_type: "[u8; 32]".to_string(),
                },
            ],
        }]
    }

    fn generate_types() -> Vec<IdlType> {
        vec![]
    }

    fn generate_errors() -> Vec<IdlError> {
        vec![IdlError {
            code: 6000,
            name: "Unauthorized".to_string(),
            msg: "Unauthorized access".to_string(),
        }]
    }
}
