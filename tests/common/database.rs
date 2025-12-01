//! Database test fixtures and utilities
//!
//! Provides utilities for setting up test databases, running migrations,
//! and cleaning up test data.

#[cfg(feature = "ssr")]
use sqlx::{PgPool, Postgres, Transaction};

/// Create a test database connection pool
///
/// This function creates a connection pool for testing. It uses the
/// DATABASE_URL environment variable or a default test database URL.
#[cfg(feature = "ssr")]
pub async fn create_test_pool() -> PgPool {
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://postgres:postgres@localhost:5432/xfcollab_test".to_string()
    });

    PgPool::connect(&database_url)
        .await
        .expect("Failed to create test database pool")
}

/// Run database migrations for testing
#[cfg(feature = "ssr")]
pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::migrate!("./migrations").run(pool).await
}

/// Clean up test data from the database
///
/// This function removes all test data while preserving the schema.
#[cfg(feature = "ssr")]
pub async fn cleanup_test_data(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("TRUNCATE TABLE messages, version_history, users, usage_tracking CASCADE")
        .execute(pool)
        .await?;
    Ok(())
}

/// Create a test transaction
///
/// Returns a transaction that will be rolled back after the test,
/// ensuring test isolation.
#[cfg(feature = "ssr")]
pub async fn create_test_transaction(pool: &PgPool) -> Transaction<'_, Postgres> {
    pool.begin()
        .await
        .expect("Failed to create test transaction")
}

/// Test database fixture
///
/// This struct manages a test database connection and automatically
/// cleans up after tests.
#[cfg(feature = "ssr")]
pub struct TestDatabase {
    pool: PgPool,
}

#[cfg(feature = "ssr")]
impl TestDatabase {
    /// Create a new test database fixture
    pub async fn new() -> Self {
        let pool = create_test_pool().await;
        run_migrations(&pool)
            .await
            .expect("Failed to run migrations");
        Self { pool }
    }

    /// Get the database pool
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Clean up test data
    pub async fn cleanup(&self) -> Result<(), sqlx::Error> {
        cleanup_test_data(&self.pool).await
    }
}

#[cfg(feature = "ssr")]
impl Drop for TestDatabase {
    fn drop(&mut self) {
        // Cleanup happens automatically when pool is dropped
    }
}

#[cfg(not(feature = "ssr"))]
pub fn create_test_pool() {
    // Placeholder for non-SSR builds
}

#[cfg(not(feature = "ssr"))]
pub fn run_migrations(_pool: &()) {
    // Placeholder for non-SSR builds
}

#[cfg(not(feature = "ssr"))]
pub fn cleanup_test_data(_pool: &()) {
    // Placeholder for non-SSR builds
}
