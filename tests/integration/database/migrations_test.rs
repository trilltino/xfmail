//! Database migration tests
//!
//! Tests to ensure migrations run correctly and database schema is valid

#[cfg(feature = "ssr")]
mod tests {
    use tests::common::database::{create_test_pool, run_migrations};

    #[tokio::test]
    async fn test_migrations_run_successfully() {
        let pool = create_test_pool().await;
        let result = run_migrations(&pool).await;
        assert!(result.is_ok(), "Migrations should run successfully");
    }

    #[tokio::test]
    async fn test_users_table_exists() {
        let pool = create_test_pool().await;
        run_migrations(&pool).await.unwrap();

        let result = sqlx::query("SELECT 1 FROM users LIMIT 1")
            .execute(&pool)
            .await;

        assert!(result.is_ok(), "Users table should exist");
    }

    #[tokio::test]
    async fn test_messages_table_exists() {
        let pool = create_test_pool().await;
        run_migrations(&pool).await.unwrap();

        let result = sqlx::query("SELECT 1 FROM messages LIMIT 1")
            .execute(&pool)
            .await;

        assert!(result.is_ok(), "Messages table should exist");
    }
}
