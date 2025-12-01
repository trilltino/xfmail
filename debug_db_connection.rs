use sqlx::postgres::PgPool;
use sqlx::Row;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Read database URL from environment
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:Ab13cba46def79_@localhost:5433/xfchat".to_string());
    
    println!("🔍 DEBUGGING DATABASE CONNECTION");
    println!("================================");
    println!("DATABASE_URL: {}", database_url);
    
    // Parse the URL to get components
    let url_parts: Vec<&str> = database_url.split('/').collect();
    let base_url = url_parts[..url_parts.len()-1].join("/");
    let db_name = url_parts.last().unwrap();
    
    println!("Base URL: {}", base_url);
    println!("Database name: {}", db_name);
    
    // Test connection to base server
    println!("\n🧪 Testing connection to base server...");
    match PgPool::connect(&base_url).await {
        Ok(pool) => {
            println!("✅ Base server connection successful!");
            
            // Test if database exists
            println!("\n🔍 Checking if database '{}' exists...", db_name);
            match sqlx::query("SELECT 1 FROM pg_database WHERE datname = $1")
                .bind(db_name)
                .fetch_optional(&pool)
                .await
            {
                Ok(Some(_)) => {
                    println!("✅ Database '{}' exists!", db_name);
                    
                    // Try connecting to the specific database
                    println!("\n🔗 Testing connection to database '{}'...", db_name);
                    match PgPool::connect(&database_url).await {
                        Ok(db_pool) => {
                            println!("✅ Database connection successful!");
                            
                            // Test a simple query
                            println!("\n🧪 Testing database query...");
                            match sqlx::query("SELECT current_database(), current_user, version()")
                                .fetch_one(&db_pool)
                                .await
                            {
                                Ok(row) => {
                                    let db: String = row.get("current_database");
                                    let user: String = row.get("current_user");
                                    let version: String = row.get("version");
                                    
                                    println!("✅ Query successful!");
                                    println!("📊 Current Database: {}", db);
                                    println!("👤 Current User: {}", user);
                                    println!("🔧 PostgreSQL Version: {}", version);
                                    
                                    // Check if users table exists
                                    println!("\n📋 Checking if 'users' table exists...");
                                    match sqlx::query("SELECT count(*) FROM information_schema.tables WHERE table_name = 'users'")
                                        .fetch_one(&db_pool)
                                        .await
                                    {
                                        Ok(table_row) => {
                                            let count: i64 = table_row.get(0);
                                            if count > 0 {
                                                println!("✅ Users table exists!");
                                            } else {
                                                println!("⚠️  Users table does not exist - you'll need to run migrations");
                                            }
                                        }
                                        Err(e) => println!("❌ Error checking users table: {}", e),
                                    }
                                }
                                Err(e) => {
                                    println!("❌ Database query failed: {}", e);
                                }
                            }
                        }
                        Err(e) => {
                            println!("❌ Database connection failed: {}", e);
                            println!("💡 This usually means the database exists but connection failed");
                            println!("💡 Check credentials and permissions");
                        }
                    }
                }
                Ok(None) => {
                    println!("❌ Database '{}' does not exist!", db_name);
                    println!("💡 You need to create the database first");
                }
                Err(e) => {
                    println!("❌ Error checking database existence: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Base server connection failed: {}", e);
            println!("💡 Possible issues:");
            println!("   - Wrong host/port in DATABASE_URL");
            println!("   - Wrong credentials");
            println!("   - PostgreSQL server not running on that port");
            println!("   - Network/firewall issues");
        }
    }
    
    println!("\n🎯 SUMMARY:");
    println!("- Check if PostgreSQL is running on port 5433");
    println!("- Verify credentials: postgres / Ab13cba46def79_");
    println!("- Ensure database '{}' was created in the right PostgreSQL instance", db_name);
    
    Ok(())
}