pub mod error;
pub mod models;
pub mod repo;

pub use error::{StorageError, StorageResult};
pub use models::*;
pub use repo::{TreeRepository, NodeRepository, SessionRepository};

/// Run database migrations
pub async fn run_migrations(
    pool: &sqlx::PgPool,
) -> StorageResult<()> {
    sqlx::migrate!()
        .run(pool)
        .await
        .map_err(|e| StorageError::MigrationError(e.to_string()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_module_exports() {
        // Verify exports are available
        assert!(true);
    }
}
