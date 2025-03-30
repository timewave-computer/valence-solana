use anchor_lang::prelude::*;

#[error_code]
pub enum StorageAccountError {
    #[msg("The operation requires owner authority")]
    UnauthorizedOwnerOperation,
    #[msg("The provided key already exists")]
    KeyAlreadyExists,
    #[msg("The provided key does not exist")]
    KeyNotFound,
    #[msg("The provided value type does not match")]
    ValueTypeMismatch,
    #[msg("The provided value is invalid or corrupted")]
    InvalidValue,
    #[msg("The size of the value is too large")]
    ValueTooLarge,
    #[msg("The storage item could not be created")]
    StorageItemCreationFailed,
    #[msg("The storage item could not be updated")]
    StorageItemUpdateFailed,
    #[msg("The storage item could not be deleted")]
    StorageItemDeletionFailed,
    #[msg("The batch update operation failed")]
    BatchUpdateFailed,
    #[msg("The operation requires storage authority")]
    UnauthorizedStorageOperation,
    #[msg("The provided address does not match the expected storage account")]
    InvalidStorageAccount,
    #[msg("The storage capacity has been exceeded")]
    StorageCapacityExceeded,
} 