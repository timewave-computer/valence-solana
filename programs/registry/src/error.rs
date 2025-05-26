use anchor_lang::prelude::*;

#[error_code]
pub enum RegistryError {
    #[msg("You are not authorized to perform this operation")]
    NotAuthorized,
    
    #[msg("Library type string is too long")]
    LibraryTypeTooLong,
    
    #[msg("Description string is too long")]
    DescriptionTooLong,
    
    #[msg("Library not found in registry")]
    LibraryNotFound,
    
    #[msg("Library is not approved")]
    LibraryNotApproved,
    
    #[msg("Invalid authorization program ID")]
    InvalidAuthorizationProgram,
    
    #[msg("Invalid account factory program ID")]
    InvalidAccountFactory,
    
    #[msg("Invalid version string")]
    InvalidVersionString,
    
    #[msg("Library already registered")]
    LibraryAlreadyRegistered,
    
    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,
    
    #[msg("Dependency already exists")]
    DependencyAlreadyExists,
    
    #[msg("Dependency not found")]
    DependencyNotFound,
    
    #[msg("Circular dependency detected")]
    CircularDependency,
    
    #[msg("Invalid dependency version")]
    InvalidDependencyVersion,
    
    #[msg("Dependency graph too large")]
    DependencyGraphTooLarge,
} 