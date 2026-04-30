use thiserror::Error;

/// Storage layer errors
#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Tree not found: {0}")]
    TreeNotFound(uuid::Uuid),

    #[error("Node not found: {0}")]
    NodeNotFound(uuid::Uuid),

    #[error("Slug already exists: {0}")]
    SlugConflict(String),

    #[error("Invalid tree state: {0}")]
    InvalidState(String),

    #[error("Validation error: {0}")]
    ValidationError(String),

    #[error("Migration error: {0}")]
    MigrationError(String),
}

/// Result type for storage operations
pub type StorageResult<T> = Result<T, StorageError>;
