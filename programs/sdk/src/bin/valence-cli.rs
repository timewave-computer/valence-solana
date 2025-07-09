/// Valence Protocol CLI Tool
/// 
/// A command-line interface for interacting with the Valence Protocol
/// using the Valence Rust SDK.

use clap::{Parser, Subcommand, ValueEnum};
use valence_sdk::*;
use solana_sdk::signature::Keypair;
use std::path::PathBuf;
use tokio;

#[derive(Parser)]
#[command(name = "valence-cli")]
#[command(about = "A command-line interface for Valence Protocol")]
#[command(version = "0.1.0")]
struct Cli {
    /// RPC endpoint URL
    #[arg(long, env = "VALENCE_RPC_URL")]
    rpc_url: Option<String>,
    
    /// Cluster to connect to
    #[arg(long, value_enum, default_value_t = Cluster::Localnet)]
    cluster: Cluster,
    
    /// Path to keypair file
    #[arg(long, env = "VALENCE_KEYPAIR")]
    keypair: Option<PathBuf>,
    
    /// Commitment level
    #[arg(long, value_enum, default_value_t = Commitment::Confirmed)]
    commitment: Commitment,
    
    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
    
    #[command(subcommand)]
    command: Commands,
}

#[derive(Clone, ValueEnum)]
enum Cluster {
    Mainnet,
    Testnet,
    Devnet,
    Localnet,
}

#[derive(Clone, ValueEnum)]
enum Commitment {
    Processed,
    Confirmed,
    Finalized,
}

#[derive(Subcommand)]
enum Commands {
    /// Program management commands
    Program {
        #[command(subcommand)]
        command: ProgramCommands,
    },
    /// Capability management commands
    Capability {
        #[command(subcommand)]
        command: CapabilityCommands,
    },
    /// Session management commands
    Session {
        #[command(subcommand)]
        command: SessionCommands,
    },
    /// Library registry commands
    Library {
        #[command(subcommand)]
        command: LibraryCommands,
    },
    /// Utility commands
    Utils {
        #[command(subcommand)]
        command: UtilCommands,
    },
}

#[derive(Subcommand)]
enum ProgramCommands {
    /// Initialize the processor singleton
    InitProcessor {
        /// Authority keypair
        #[arg(long)]
        authority: Option<PathBuf>,
    },
    /// Initialize the scheduler singleton
    InitScheduler {
        /// Authority keypair
        #[arg(long)]
        authority: Option<PathBuf>,
        /// Maximum number of shards
        #[arg(long, default_value = "100")]
        max_shards: u16,
        /// Maximum queue size
        #[arg(long, default_value = "1000")]
        max_queue_size: u16,
    },
    /// Initialize the diff singleton
    InitDiff {
        /// Authority keypair
        #[arg(long)]
        authority: Option<PathBuf>,
        /// Maximum batch size
        #[arg(long, default_value = "50")]
        max_batch_size: u16,
    },
    /// Initialize the shard program
    InitShard {
        /// Authority keypair
        #[arg(long)]
        authority: Option<PathBuf>,
        /// Program ID
        #[arg(long)]
        program_id: String,
    },
    /// Initialize the registry program
    InitRegistry {
        /// Authority keypair
        #[arg(long)]
        authority: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum CapabilityCommands {
    /// Grant a new capability
    Grant {
        /// Authority keypair
        #[arg(long)]
        authority: Option<PathBuf>,
        /// Shard state address
        #[arg(long)]
        shard_state: String,
        /// Capability ID
        #[arg(long)]
        capability_id: String,
        /// Verification function names (comma-separated)
        #[arg(long)]
        verification_functions: String,
        /// Description
        #[arg(long)]
        description: String,
    },
    /// Update an existing capability
    Update {
        /// Authority keypair
        #[arg(long)]
        authority: Option<PathBuf>,
        /// Shard state address
        #[arg(long)]
        shard_state: String,
        /// Capability address
        #[arg(long)]
        capability: String,
        /// New verification function names (comma-separated)
        #[arg(long)]
        verification_functions: String,
        /// New description
        #[arg(long)]
        description: Option<String>,
    },
    /// Revoke a capability
    Revoke {
        /// Authority keypair
        #[arg(long)]
        authority: Option<PathBuf>,
        /// Shard state address
        #[arg(long)]
        shard_state: String,
        /// Capability address
        #[arg(long)]
        capability: String,
    },
    /// Execute a capability
    Execute {
        /// Capability ID
        #[arg(long)]
        capability_id: String,
        /// Session address
        #[arg(long)]
        session: String,
        /// Caller keypair
        #[arg(long)]
        caller: Option<PathBuf>,
        /// Input data (hex-encoded)
        #[arg(long)]
        input_data: Option<String>,
        /// Compute limit
        #[arg(long)]
        compute_limit: Option<u32>,
        /// Max execution time (seconds)
        #[arg(long)]
        max_time: Option<u64>,
    },
    /// Get capability information
    Get {
        /// Shard state address
        #[arg(long)]
        shard_state: String,
        /// Capability ID
        #[arg(long)]
        capability_id: String,
    },
    /// List capabilities with templates
    Templates,
    /// Create capability from template
    FromTemplate {
        /// Authority keypair
        #[arg(long)]
        authority: Option<PathBuf>,
        /// Shard state address
        #[arg(long)]
        shard_state: String,
        /// Capability ID
        #[arg(long)]
        capability_id: String,
        /// Template type
        #[arg(long, value_enum)]
        template: CapabilityTemplateType,
        /// Custom parameters (key=value pairs, comma-separated)
        #[arg(long)]
        parameters: Option<String>,
    },
}

#[derive(Clone, ValueEnum)]
enum CapabilityTemplateType {
    BasicPermission,
    TokenTransfer,
    ZkProof,
    Custom,
}

#[derive(Subcommand)]
enum SessionCommands {
    /// Create a new session
    Create {
        /// Owner keypair
        #[arg(long)]
        owner: Option<PathBuf>,
        /// Session ID
        #[arg(long)]
        session_id: String,
        /// Capabilities (comma-separated)
        #[arg(long)]
        capabilities: String,
        /// Namespaces (comma-separated)
        #[arg(long)]
        namespaces: Option<String>,
        /// Description
        #[arg(long)]
        description: String,
        /// Tags (comma-separated)
        #[arg(long)]
        tags: Option<String>,
        /// Max lifetime (seconds)
        #[arg(long)]
        max_lifetime: Option<i64>,
    },
    /// Get session information
    Get {
        /// Session ID
        #[arg(long)]
        session_id: String,
    },
    /// List sessions for an owner
    List {
        /// Owner address
        #[arg(long)]
        owner: String,
        /// Only active sessions
        #[arg(long)]
        active_only: bool,
    },
    /// Execute capability in session context
    Execute {
        /// Session ID
        #[arg(long)]
        session_id: String,
        /// Capability ID
        #[arg(long)]
        capability_id: String,
        /// Caller keypair
        #[arg(long)]
        caller: Option<PathBuf>,
        /// Input data (hex-encoded)
        #[arg(long)]
        input_data: Option<String>,
    },
    /// List session templates
    Templates,
    /// Create session from template
    FromTemplate {
        /// Owner keypair
        #[arg(long)]
        owner: Option<PathBuf>,
        /// Session ID
        #[arg(long)]
        session_id: String,
        /// Template type
        #[arg(long, value_enum)]
        template: SessionTemplateType,
        /// Custom parameters (key=value pairs, comma-separated)
        #[arg(long)]
        parameters: Option<String>,
    },
}

#[derive(Clone, ValueEnum)]
enum SessionTemplateType {
    Basic,
    Finance,
    ZkProof,
    Custom,
}

#[derive(Subcommand)]
enum LibraryCommands {
    /// Register a new library
    Register {
        /// Authority keypair
        #[arg(long)]
        authority: Option<PathBuf>,
        /// Library ID
        #[arg(long)]
        library_id: String,
        /// Library name
        #[arg(long)]
        name: String,
        /// Version
        #[arg(long)]
        version: String,
        /// Program ID
        #[arg(long)]
        program_id: String,
        /// Tags (comma-separated)
        #[arg(long)]
        tags: Option<String>,
    },
    /// Query a library
    Get {
        /// Library ID
        #[arg(long)]
        library_id: String,
    },
    /// List libraries
    List {
        /// Page number
        #[arg(long, default_value = "1")]
        page: u64,
        /// Page size
        #[arg(long, default_value = "10")]
        page_size: u64,
        /// Filter by status
        #[arg(long)]
        status: Option<String>,
        /// Filter by tags (comma-separated)
        #[arg(long)]
        tags: Option<String>,
    },
    /// Update library status
    UpdateStatus {
        /// Authority keypair
        #[arg(long)]
        authority: Option<PathBuf>,
        /// Library ID
        #[arg(long)]
        library_id: String,
        /// New status
        #[arg(long, value_enum)]
        status: LibraryStatusCli,
    },
}

#[derive(Clone, ValueEnum)]
enum LibraryStatusCli {
    Draft,
    Published,
    Deprecated,
    Archived,
}

#[derive(Subcommand)]
enum UtilCommands {
    /// Generate a new keypair
    GenerateKeypair {
        /// Output file path
        #[arg(long)]
        output: Option<PathBuf>,
    },
    /// Validate a capability ID
    ValidateCapabilityId {
        /// Capability ID to validate
        capability_id: String,
    },
    /// Validate a version string
    ValidateVersion {
        /// Version to validate
        version: String,
    },
    /// Calculate metadata hash
    CalculateHash {
        /// Name
        #[arg(long)]
        name: String,
        /// Version
        #[arg(long)]
        version: String,
        /// Description
        #[arg(long)]
        description: String,
        /// Tags (comma-separated)
        #[arg(long)]
        tags: Option<String>,
    },
    /// Get current timestamp
    Timestamp,
    /// Convert pubkey formats
    ConvertPubkey {
        /// Public key to convert
        pubkey: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    // Initialize logging
    if cli.verbose {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug")).init();
    } else {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    }
    
    // Create client configuration
    let config = create_config(&cli).await?;
    let client = ValenceClient::new(config)?;
    
    // Execute command
    match cli.command {
        Commands::Program { command } => handle_program_commands(command, &client).await?,
        Commands::Capability { command } => handle_capability_commands(command, &client).await?,
        Commands::Session { command } => handle_session_commands(command, &client).await?,
        Commands::Library { command } => handle_library_commands(command, &client).await?,
        Commands::Utils { command } => handle_util_commands(command).await?,
    }
    
    Ok(())
}

async fn create_config(cli: &Cli) -> Result<ValenceConfig, Box<dyn std::error::Error>> {
    // Determine cluster
    let cluster = match cli.cluster {
        Cluster::Mainnet => anchor_client::Cluster::Mainnet,
        Cluster::Testnet => anchor_client::Cluster::Testnet,
        Cluster::Devnet => anchor_client::Cluster::Devnet,
        Cluster::Localnet => anchor_client::Cluster::Localnet,
    };
    
    // Load keypair
    let payer = if let Some(keypair_path) = &cli.keypair {
        load_keypair_from_file(&keypair_path.to_string_lossy())?
    } else {
        // Try to load from default Solana config
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let default_path = format!("{}/.config/solana/id.json", home);
        load_keypair_from_file(&default_path).unwrap_or_else(|_| {
            println!("Warning: Could not load keypair, generating temporary one");
            Keypair::new()
        })
    };
    
    // Determine commitment
    let commitment = match cli.commitment {
        Commitment::Processed => CommitmentConfig::processed(),
        Commitment::Confirmed => CommitmentConfig::confirmed(),
        Commitment::Finalized => CommitmentConfig::finalized(),
    };
    
    Ok(ValenceConfig {
        program_ids: ProgramIds::default(), // In practice, these would be configured
        cluster,
        payer,
        commitment: Some(commitment),
    })
}

async fn handle_program_commands(
    command: ProgramCommands,
    client: &ValenceClient,
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        ProgramCommands::InitEval { authority, shard_address } => {
            let authority_keypair = load_authority_keypair(authority)?;
            let shard_pubkey = shard_address.parse()?;
            
            println!("Initializing eval program...");
            let signature = client.initialize_eval(&authority_keypair.pubkey(), &shard_pubkey).await?;
            println!("✅ Eval program initialized: {}", signature);
        }
        ProgramCommands::InitShard { authority, program_id, eval_address } => {
            let authority_keypair = load_authority_keypair(authority)?;
            let program_pubkey = program_id.parse()?;
            let eval_pubkey = eval_address.parse()?;
            
            println!("Initializing shard program...");
            let signature = client.initialize_shard(&authority_keypair.pubkey(), &program_pubkey, &eval_pubkey).await?;
            println!("✅ Shard program initialized: {}", signature);
        }
        ProgramCommands::InitRegistry { authority } => {
            let authority_keypair = load_authority_keypair(authority)?;
            
            println!("Initializing registry program...");
            let signature = client.initialize_registry(&authority_keypair.pubkey()).await?;
            println!("✅ Registry program initialized: {}", signature);
        }
    }
    Ok(())
}

async fn handle_capability_commands(
    command: CapabilityCommands,
    client: &ValenceClient,
) -> Result<(), Box<dyn std::error::Error>> {
    let capability_manager = client.capability_manager();
    
    match command {
        CapabilityCommands::Grant { authority, shard_state, capability_id, verification_functions, description } => {
            let authority_keypair = load_authority_keypair(authority)?;
            let shard_pubkey = shard_state.parse()?;
            let vf_hashes = parse_verification_functions(&verification_functions, &capability_manager)?;
            
            println!("Granting capability '{}'...", capability_id);
            let signature = capability_manager.grant_capability(
                &authority_keypair.pubkey(),
                &shard_pubkey,
                &capability_id,
                vf_hashes,
                &description,
            ).await?;
            println!("✅ Capability granted: {}", signature);
        }
        CapabilityCommands::Execute { capability_id, session, caller, input_data, compute_limit, max_time } => {
            let caller_keypair = load_authority_keypair(caller)?;
            let session_pubkey = session.parse()?;
            let input = parse_hex_data(input_data)?;
            
            let context = ValenceExecutionContext::new(
                capability_id.clone(),
                session_pubkey,
                caller_keypair.pubkey(),
            ).with_input_data(input);
            
            let mut config = ExecutionConfig::default();
            if let Some(limit) = compute_limit {
                config.max_compute_units = Some(limit);
            }
            if let Some(time) = max_time {
                config.max_execution_time = Some(time);
            }
            
            println!("Executing capability '{}'...", capability_id);
            let result = capability_manager.execute_capability(&context, &config).await?;
            println!("✅ Capability executed: {}", result.transaction_result.signature);
        }
        CapabilityCommands::Get { shard_state, capability_id } => {
            let shard_pubkey = shard_state.parse()?;
            
            if let Some(capability) = capability_manager.get_capability(&shard_pubkey, &capability_id).await? {
                println!("Capability Information:");
                println!("  ID: {}", capability.capability_id);
                println!("  Shard: {}", capability.shard);
                println!("  Description: {}", capability.description);
                println!("  Active: {}", capability.is_active);
                println!("  Total Executions: {}", capability.total_executions);
                println!("  Verification Functions: {}", capability.verification_functions.len());
            } else {
                println!("❌ Capability not found");
            }
        }
        CapabilityCommands::Templates => {
            println!("Available Capability Templates:");
            println!("  basic-permission: Basic permission-based capability");
            println!("  token-transfer: Token transfer capability");
            println!("  zk-proof: Zero-knowledge proof verification capability");
            println!("  custom: Custom capability template");
        }
        CapabilityCommands::FromTemplate { authority, shard_state, capability_id, template, parameters } => {
            let authority_keypair = load_authority_keypair(authority)?;
            let shard_pubkey = shard_state.parse()?;
            let template_type = match template {
                CapabilityTemplateType::BasicPermission => crate::CapabilityTemplateType::BasicPermission,
                CapabilityTemplateType::TokenTransfer => crate::CapabilityTemplateType::TokenTransfer,
                CapabilityTemplateType::ZkProof => crate::CapabilityTemplateType::ZkProof,
                CapabilityTemplateType::Custom => crate::CapabilityTemplateType::Custom,
            };
            
            let template = capability_manager.create_capability_template(template_type);
            let custom_params = parse_parameters(parameters);
            
            println!("Creating capability from template...");
            let signature = capability_manager.create_capability_from_template(
                &authority_keypair.pubkey(),
                &shard_pubkey,
                &capability_id,
                &template,
                custom_params,
            ).await?;
            println!("✅ Capability created from template: {}", signature);
        }
        _ => println!("Command not yet implemented"),
    }
    Ok(())
}

async fn handle_session_commands(
    command: SessionCommands,
    client: &ValenceClient,
) -> Result<(), Box<dyn std::error::Error>> {
    let session_manager = client.session_manager();
    
    match command {
        SessionCommands::Create { owner, session_id, capabilities, namespaces, description, tags, max_lifetime } => {
            let owner_keypair = load_authority_keypair(owner)?;
            let capability_list = capabilities.split(',').map(|s| s.trim().to_string()).collect();
            let namespace_list = namespaces.map(|ns| ns.split(',').map(|s| s.trim().to_string()).collect()).unwrap_or_default();
            let tag_list = tags.map(|t| t.split(',').map(|s| s.trim().to_string()).collect()).unwrap_or_default();
            
            let metadata = SessionMetadata {
                description,
                tags: tag_list,
                max_lifetime: max_lifetime.unwrap_or(0),
            };
            
            println!("Creating session '{}'...", session_id);
            let session = session_manager.create_session(
                &owner_keypair.pubkey(),
                &session_id,
                capability_list,
                namespace_list,
                metadata,
            ).await?;
            
            println!("✅ Session created:");
            println!("  ID: {}", session.session_id);
            println!("  Owner: {}", session.owner);
            println!("  Capabilities: {}", session.capabilities.join(", "));
            println!("  Namespaces: {}", session.namespaces.join(", "));
        }
        SessionCommands::Templates => {
            println!("Available Session Templates:");
            println!("  basic: Basic session with standard capabilities");
            println!("  finance: Session for financial operations");
            println!("  zk-proof: Session for zero-knowledge proof operations");
            println!("  custom: Custom session template");
        }
        _ => println!("Command not yet implemented"),
    }
    Ok(())
}

async fn handle_library_commands(
    command: LibraryCommands,
    client: &ValenceClient,
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        LibraryCommands::Register { authority, library_id, name, version, program_id, tags } => {
            let authority_keypair = load_authority_keypair(authority)?;
            let program_pubkey = program_id.parse()?;
            let tag_list = tags.map(|t| t.split(',').map(|s| s.trim().to_string()).collect()).unwrap_or_default();
            
            let library_entry = LibraryEntry {
                library_id: library_id.clone(),
                name: name.clone(),
                version: version.clone(),
                author: authority_keypair.pubkey(),
                metadata_hash: calculate_metadata_hash(&name, &version, "Library description", &tag_list),
                program_id: program_pubkey,
                status: LibraryStatus::Published,
                dependencies: vec![],
                tags: tag_list,
                is_verified: false,
                usage_count: 0,
            };
            
            println!("Registering library '{}'...", library_id);
            let signature = client.register_library(&authority_keypair.pubkey(), &library_entry).await?;
            println!("✅ Library registered: {}", signature);
        }
        LibraryCommands::Get { library_id } => {
            if let Some(library) = client.query_library(&library_id).await? {
                println!("Library Information:");
                println!("  ID: {}", library.library_id);
                println!("  Name: {}", library.name);
                println!("  Version: {}", library.version);
                println!("  Author: {}", library.author);
                println!("  Status: {:?}", library.status);
                println!("  Tags: {}", library.tags.join(", "));
                println!("  Verified: {}", library.is_verified);
                println!("  Usage Count: {}", library.usage_count);
            } else {
                println!("❌ Library not found");
            }
        }
        _ => println!("Command not yet implemented"),
    }
    Ok(())
}

async fn handle_util_commands(command: UtilCommands) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        UtilCommands::GenerateKeypair { output } => {
            let keypair = generate_keypair();
            
            if let Some(path) = output {
                save_keypair_to_file(&keypair, &path.to_string_lossy())?;
                println!("✅ Keypair saved to: {}", path.display());
            } else {
                println!("Generated Keypair:");
                println!("  Public Key: {}", keypair.pubkey());
                println!("  Secret Key: [hidden - use --output to save]");
            }
        }
        UtilCommands::ValidateCapabilityId { capability_id } => {
            match validate_capability_id(&capability_id) {
                Ok(_) => println!("✅ Capability ID '{}' is valid", capability_id),
                Err(e) => println!("❌ Invalid capability ID: {}", e),
            }
        }
        UtilCommands::ValidateVersion { version } => {
            match validate_version(&version) {
                Ok(_) => println!("✅ Version '{}' is valid", version),
                Err(e) => println!("❌ Invalid version: {}", e),
            }
        }
        UtilCommands::CalculateHash { name, version, description, tags } => {
            let tag_list = tags.map(|t| t.split(',').map(|s| s.trim().to_string()).collect()).unwrap_or_default();
            let hash = calculate_metadata_hash(&name, &version, &description, &tag_list);
            println!("Metadata Hash: {}", hex::encode(hash));
        }
        UtilCommands::Timestamp => {
            let timestamp = current_timestamp();
            println!("Current Timestamp: {} ({})", timestamp, timestamp_to_string(timestamp));
        }
        UtilCommands::ConvertPubkey { pubkey } => {
            match string_to_pubkey(&pubkey) {
                Ok(pk) => {
                    println!("Public Key: {}", pk);
                    println!("Base58: {}", pubkey_to_string(&pk));
                    println!("Bytes: {:?}", pk.to_bytes());
                }
                Err(e) => println!("❌ Invalid public key: {}", e),
            }
        }
    }
    Ok(())
}

// Helper functions

fn load_authority_keypair(path: Option<PathBuf>) -> Result<Keypair, Box<dyn std::error::Error>> {
    if let Some(keypair_path) = path {
        Ok(load_keypair_from_file(&keypair_path.to_string_lossy())?)
    } else {
        // Use the same keypair as the client
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let default_path = format!("{}/.config/solana/id.json", home);
        Ok(load_keypair_from_file(&default_path).unwrap_or_else(|_| Keypair::new()))
    }
}

fn parse_verification_functions(
    functions: &str,
    manager: &CapabilityManager,
) -> Result<Vec<[u8; 32]>, Box<dyn std::error::Error>> {
    let function_names: Vec<&str> = functions.split(',').map(|s| s.trim()).collect();
    let known_functions = manager.get_verification_functions();
    let mut result = Vec::new();
    
    for name in function_names {
        if let Some(hash) = known_functions.get(name) {
            result.push(*hash);
        } else {
            return Err(format!("Unknown verification function: {}", name).into());
        }
    }
    
    Ok(result)
}

fn parse_hex_data(data: Option<String>) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    if let Some(hex_str) = data {
        let hex_str = hex_str.strip_prefix("0x").unwrap_or(&hex_str);
        Ok(hex::decode(hex_str)?)
    } else {
        Ok(Vec::new())
    }
}

fn parse_parameters(params: Option<String>) -> Option<std::collections::HashMap<String, String>> {
    params.map(|p| {
        p.split(',')
            .filter_map(|pair| {
                let mut parts = pair.splitn(2, '=');
                match (parts.next(), parts.next()) {
                    (Some(key), Some(value)) => Some((key.trim().to_string(), value.trim().to_string())),
                    _ => None,
                }
            })
            .collect()
    })
} 