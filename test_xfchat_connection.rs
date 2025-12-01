// XFChat Database Connection Test
// This script tests the PostgreSQL connection for the XFChat database
// Compile with: rustc --extern sqlx=path/to/sqlx test_xfchat_connection.rs

use std::env;
use sqlx::{postgres::PgPoolOptions, PgPool};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔗 Testing XFChat Database Connection...");
    
    // Get database URL from environment
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:Ab13cba46def79_@localhost:5432/xfchat".to_string());
    
    println!("📡 Connection URL: {}", database_url.replace("Ab13cba46def79_", "*******"));
    
    // Create connection pool
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(10))
        .connect(&database_url)
        .await?;
    
    println!("✅ Database connection successful!");
    
    // Test basic query
    let result: (i64,) = sqlx::query_as("SELECT COUNT(*) as count FROM users")
        .fetch_one(&pool)
        .await?;
    
    println!("📊 Current user count in XFChat database: {}", result.0);
    
    // Test extensions
    let extensions: Vec<String> = sqlx::query_scalar("SELECT extname FROM pg_extension WHERE extname IN ('uuid-ossp', 'pgcrypto')")
        .fetch_all(&pool)
        .await?;
    
    println!("🔧 Available extensions: {:?}", extensions);
    
    // Test tables
    let tables: Vec<String> = sqlx::query_scalar("SELECT tablename FROM pg_tables WHERE schemaname = 'public' ORDER BY tablename")
        .fetch_all(&pool)
        .await?;
    
    println!("📋 Tables in XFChat database:");
    for table in tables {
        println!("  - {}", table);
    }
    
    // Close connection
    pool.close().await;
    println!("🎉 XFChat database connection test completed successfully!");
    
    Ok(())
}