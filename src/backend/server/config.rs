/**
 * Server Configuration
 * 
 * This module handles loading and validation of server configuration,
 * focusing on the optional PostgreSQL database connection.
 * 
 * # Configuration Sources
 * 
 * Configuration is loaded from environment variables, with sensible defaults
 * for local development when possible.
 * 
 * # Error Handling
 * 
 * Configuration errors are logged but do not prevent server startup.
 * Services that fail to initialize are set to `None` and the server
 * continues without them.
 */

#[cfg(feature = "ssr")]
use sqlx::PgPool;

/// Database configuration result
/// 
/// Contains the database connection pool if successfully configured,
/// or `None` if the database is not available.
#[cfg(feature = "ssr")]
pub type DatabaseConfig = Option<PgPool>;

/// Load and initialize database connection pool
/// 
/// This function:
/// 1. Reads `DATABASE_URL` from environment
/// 2. Creates a PostgreSQL connection pool
/// 3. Runs database migrations
/// 4. Optionally clears the users table (for development)
/// 
/// # Returns
/// 
/// - `Some(PgPool)` if database is successfully configured
/// - `None` if `DATABASE_URL` is not set or connection fails
/// 
/// # Errors
/// 
/// Errors are logged but do not prevent server startup. The function
/// returns `None` on any error, allowing the server to run without
/// database features.
/// 
/// # Example
/// 
/// ```rust
/// use braid_site::backend::server::config::load_database;
/// 
/// let db_pool = load_database().await;
/// if let Some(pool) = &db_pool {
///     // Database is available
/// } else {
///     // Database features disabled
/// }
/// ```
#[cfg(feature = "ssr")]
pub async fn load_database() -> DatabaseConfig {
    let database_url = match std::env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            tracing::warn!("DATABASE_URL not set. Database features will be disabled.");
            return None;
        }
    };
    
    tracing::info!("Connecting to database...");
    
    let pool = match PgPool::connect(&database_url).await {
        Ok(pool) => pool,
        Err(e) => {
            tracing::error!("Failed to create database connection pool: {:?}", e);
            tracing::warn!("Database features will be disabled.");
            return None;
        }
    };
    
    tracing::info!("Database connection pool created successfully");
    
    // Run migrations
    tracing::info!("Running database migrations...");
    match sqlx::migrate!().run(&pool).await {
        Ok(_) => {
            tracing::info!("Database migrations completed successfully");
        }
        Err(e) => {
            tracing::error!("Failed to run database migrations: {:?}", e);
            tracing::error!("Migration error details: {}", e);
            // Continue anyway - migrations might have already been run
            tracing::warn!("Continuing without migrations - database might not be up to date");
        }
    }
    
    Some(pool)
}
