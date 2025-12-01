# PostgreSQL Exclusive Implementation Guide (XFChat)

## Detailed Code Changes Reference

This document provides specific code changes needed for each file in the PostgreSQL migration for XFChat project.

---

## 1. Database Configuration Changes

### File: `src/backend/server/config.rs`

**Current Implementation**:
```rust
pub type DatabaseConfig = Option<PgPool>;

pub async fn load_database() -> DatabaseConfig {
    let database_url = match std::env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            tracing::warn!("DATABASE_URL not set. Database features will be disabled.");
            return None;
        }
    };
    
    let pool = match PgPool::connect(&database_url).await {
        Ok(pool) => pool,
        Err(e) => {
            tracing::error!("Failed to create database connection pool: {:?}", e);
            tracing::warn!("Database features will be disabled.");
            return None;
        }
    };
    
    match sqlx::migrate!().run(&pool).await {
        Ok(_) => {
            tracing::info!("Database migrations completed successfully");
        }
        Err(e) => {
            tracing::error!("Failed to run database migrations: {:?}", e);
            tracing::warn!("Continuing without migrations - database might not be up to date");
        }
    }
    
    Some(pool)
}
```

**Required Changes**:
1. Change return type from `Option<PgPool>` to `Result<PgPool, Box<dyn std::error::Error>>`
2. Remove all graceful degradation paths
3. Return errors instead of `None`
4. Make migrations mandatory (fail if they don't run)

**New Implementation**:
```rust
use std::error::Error;

pub type DatabaseConfig = PgPool;

#[cfg(feature = "ssr")]
pub async fn load_database() -> Result<DatabaseConfig, Box<dyn Error>> {
    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL environment variable must be set".into())?;
    
    tracing::info!("Connecting to PostgreSQL: {}", database_url);
    
    let pool = PgPool::connect(&database_url)
        .await
        .map_err(|e| format!("Failed to connect to database: {}", e))?;
    
    tracing::info!("Database connection pool created successfully");
    
    // Test connection
    sqlx::query("SELECT 1")
        .execute(&pool)
        .await
        .map_err(|e| format!("Failed to verify database connection: {}", e))?;
    
    tracing::info!("Running database migrations...");
    
    sqlx::migrate!()
        .run(&pool)
        .await
        .map_err(|e| format!("Database migrations failed: {}", e))?;
    
    tracing::info!("Database migrations completed successfully");
    
    Ok(pool)
}
```

---

## 2. Application State Changes

### File: `src/backend/server/state.rs`

**Current Implementation** (lines 213-310):
```rust
pub struct AppState {
    pub chat_state: Arc<RwLock<ChatState>>,
    pub collab_state: Arc<RwLock<CollabState>>,
    pub message_broadcast: broadcast::Sender<MessageEvent>,
    pub realtime_broadcast: RealtimeEventBroadcast,
    pub db_pool: Option<PgPool>,  // ← CHANGE THIS
    pub messaging_broadcast: MessagingBroadcastState,
    pub messaging_crdt: MessagingCrdtState,
}

impl FromRef<AppState> for Option<PgPool> {  // ← CHANGE THIS
    fn from_ref(app_state: &AppState) -> Self {
        app_state.db_pool.clone()
    }
}
```

**Required Changes**:
1. Change `db_pool: Option<PgPool>` to `db_pool: PgPool`
2. Update the `FromRef` implementation to work with required `PgPool`
3. Update documentation to reflect mandatory database

**New Implementation**:
```rust
pub struct AppState {
    /// Shared chat state containing messages and version history
    pub chat_state: Arc<RwLock<ChatState>>,
    
    /// Shared collaborative editing state
    pub collab_state: Arc<RwLock<CollabState>>,
    
    /// Broadcast channel for chat message updates
    pub message_broadcast: broadcast::Sender<MessageEvent>,
    
    /// Generic real-time event broadcast channel
    pub realtime_broadcast: RealtimeEventBroadcast,
    
    /// PostgreSQL database connection pool (required)
    ///
    /// This is guaranteed to be a valid, connected pool.
    /// All database operations should execute without Option checking.
    pub db_pool: PgPool,
    
    /// Messaging broadcast state for real-time delivery
    pub messaging_broadcast: MessagingBroadcastState,
    
    /// CRDT state for messaging conversations
    pub messaging_crdt: MessagingCrdtState,
}

impl FromRef<AppState> for PgPool {
    fn from_ref(app_state: &AppState) -> Self {
        app_state.db_pool.clone()
    }
}
```

---

## 3. Server Startup Changes

### File: `src/backend/main.rs`

**Current Implementation** (lines 30-48):
```rust
let port = std::env::var("SERVER_PORT")
    .unwrap_or_else(|_| "3000".to_string())
    .parse::<u16>()
    .unwrap_or(3000);

let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
eprintln!("[STARTUP] Starting server on {}", addr);
tracing::warn!("Starting server on {}", addr);

let listener = tokio::net::TcpListener::bind(addr).await?;
eprintln!("[STARTUP] Listening on {}", addr);
eprintln!("[STARTUP] Client should connect to http://127.0.0.1:{}", port);
axum::serve(listener, app).await?;
```

**Required Changes**:
1. Load database before creating AppState
2. Fail if database connection is not established
3. Add database health check to startup validation
4. Update startup logging

**New Implementation**:
```rust
// Load and validate database connection
eprintln!("[STARTUP] Loading database configuration...");
let db_pool = load_database()
    .await
    .expect("[STARTUP] FATAL: Failed to initialize database. Ensure DATABASE_URL is set and PostgreSQL is running.");
eprintln!("[STARTUP] Database connection established and migrations completed");

// Verify database is actually working
match sqlx::query("SELECT COUNT(*) as count FROM users")
    .fetch_one(&db_pool)
    .await
{
    Ok(row) => {
        let count: i64 = row.get("count");
        eprintln!("[STARTUP] Database verified: {} users in system", count);
    }
    Err(e) => {
        eprintln!("[STARTUP] FATAL: Database query failed: {}", e);
        return Err(format!("Database verification failed: {}", e).into());
    }
}

let port = std::env::var("SERVER_PORT")
    .unwrap_or_else(|_| "3000".to_string())
    .parse::<u16>()
    .unwrap_or(3000);

let addr = std::net::SocketAddr::from(([0, 0, 0, 0], port));
eprintln!("[STARTUP] Binding server to {}", addr);

let listener = tokio::net::TcpListener::bind(addr)
    .await
    .expect("[STARTUP] FATAL: Failed to bind to server address");

eprintln!("[STARTUP] ✓ Server listening on 0.0.0.0:{}", port);
eprintln!("[STARTUP] ✓ Client should connect to http://127.0.0.1:{}", port);
eprintln!("[STARTUP] ✓ Database: PostgreSQL (required)");
eprintln!("[STARTUP] ✓ Ready to accept connections");

axum::serve(listener, app).await?;
```

---

## 4. Message Subscription Handler Changes

### File: `src/backend/messaging/message_sync.rs`

**Current Implementation** (lines 93-157):
```rust
pub async fn handle_message_subscription(
    State(db_pool): State<Option<PgPool>>,  // ← CHANGE
    State(broadcast_state): State<MessagingBroadcastState>,
    Path(conversation_id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Sse<impl StreamExt<Item = Result<axum::response::sse::Event, Infallible>>>, StatusCode> {
    eprintln!("[GET-SUB] Subscription request for conversation: {}", conversation_id);
    
    let user_id = extract_user_id(&headers)?;
    eprintln!("[GET-SUB] User ID: {}", user_id);
    
    // Load existing messages from database if available
    let messages = if let Some(pool) = db_pool.as_ref() {  // ← REMOVE THIS
        let dev_bypass = std::env::var("DEV_AUTH_BYPASS").unwrap_or_default() == "1";
        if !dev_bypass {
            match is_user_participant_in_conversation(pool, user_id, conversation_id).await {
                Ok(is_participant) => {
                    if !is_participant {
                        tracing::warn!("[MessageSync] User {} is not a participant in conversation {}", user_id, conversation_id);
                        return Err(StatusCode::FORBIDDEN);
                    }
                }
                Err(e) => {
                    tracing::error!("[MessageSync] Failed to check participant status: {:?}", e);
                    return Err(StatusCode::INTERNAL_SERVER_ERROR);
                }
            }
        } else {
            tracing::debug!("[MessageSync] DEV_AUTH_BYPASS enabled, skipping participant check");
        }
        
        match get_messages_for_conversation(pool, conversation_id, 50, 0).await {
            Ok(msgs) => {
                tracing::info!("[MessageSync] Loaded {} messages for conversation {}", msgs.len(), conversation_id);
                msgs
            }
            Err(e) => {
                tracing::error!("[MessageSync] Failed to load messages: {:?}", e);
                Vec::new()
            }
        }
    } else {
        tracing::warn!("[MessageSync] Database pool not available, starting with no initial messages");
        Vec::new()
    };
```

**Required Changes**:
1. Change parameter from `State(Option<PgPool>)` to `State(PgPool)`
2. Remove `if let Some(pool)` guards
3. Make participant check mandatory
4. Make message loading mandatory with proper error handling

**New Implementation**:
```rust
pub async fn handle_message_subscription(
    State(db_pool): State<PgPool>,  // ← NOW REQUIRED
    State(broadcast_state): State<MessagingBroadcastState>,
    Path(conversation_id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Sse<impl StreamExt<Item = Result<axum::response::sse::Event, Infallible>>>, StatusCode> {
    eprintln!("[GET-SUB] Subscription request for conversation: {}", conversation_id);
    tracing::info!("[MessageSync] Subscription request for conversation: {}", conversation_id);
    
    let user_id = extract_user_id(&headers)?;
    eprintln!("[GET-SUB] User ID: {}", user_id);
    tracing::info!("[MessageSync] User ID: {}", user_id);
    
    // Always verify participant status
    let dev_bypass = std::env::var("DEV_AUTH_BYPASS").unwrap_or_default() == "1";
    if !dev_bypass {
        let is_participant = is_user_participant_in_conversation(&db_pool, user_id, conversation_id)
            .await
            .map_err(|e| {
                tracing::error!("[MessageSync] Failed to check participant status: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;
        
        if !is_participant {
            tracing::warn!("[MessageSync] User {} is not a participant in conversation {}", user_id, conversation_id);
            return Err(StatusCode::FORBIDDEN);
        }
    } else {
        tracing::debug!("[MessageSync] DEV_AUTH_BYPASS enabled, skipping participant check");
    }
    
    // Always load messages from database
    let messages = get_messages_for_conversation(&db_pool, conversation_id, 50, 0)
        .await
        .map_err(|e| {
            tracing::error!("[MessageSync] Failed to load messages for conversation {}: {:?}", conversation_id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    tracing::info!("[MessageSync] Loaded {} messages for conversation {}", messages.len(), conversation_id);
    eprintln!("[GET-SUB] Loaded {} messages", messages.len());
```

---

## 5. Message PUT Handler Changes

### File: `src/backend/messaging/message_sync.rs` (continued)

**Current Implementation** (lines 196-250):
```rust
pub async fn handle_message_put(
    State(db_pool): State<Option<PgPool>>,  // ← CHANGE
    State(broadcast_state): State<MessagingBroadcastState>,
    Path((conversation_id, message_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
    body: String,
) -> Result<StatusCode, StatusCode> {
    // ... code ...
    
    // Save to database if available
    if let Some(pool) = db_pool.as_ref() {  // ← REMOVE THIS
        match save_message_to_db(pool, &chat_message).await {
            Ok(_) => {
                eprintln!("[PUT-MSG] Message saved to database");
            }
            Err(e) => {
                eprintln!("[PUT-MSG] Failed to save message to database: {:?}", e);
                tracing::error!("Failed to save message: {:?}", e);
            }
        }
    } else {
        tracing::warn!("Database pool not available, message not persisted");
    }
```

**Required Changes**:
1. Change parameter from `State(Option<PgPool>)` to `State<PgPool>`
2. Make message persistence mandatory
3. Return error if message cannot be saved
4. Remove fallback behavior

**New Implementation**:
```rust
pub async fn handle_message_put(
    State(db_pool): State<PgPool>,  // ← NOW REQUIRED
    State(broadcast_state): State<MessagingBroadcastState>,
    Path((conversation_id, message_id)): Path<(Uuid, Uuid)>,
    headers: HeaderMap,
    body: String,
) -> Result<StatusCode, StatusCode> {
    eprintln!("[PUT-MSG] Received message PUT request");
    
    // ... parse and validate message ...
    
    // Always persist to database
    save_message_to_db(&db_pool, &chat_message)
        .await
        .map_err(|e| {
            eprintln!("[PUT-MSG] Failed to save message to database: {:?}", e);
            tracing::error!("[MessageSync] Database save failed for message {}: {:?}", message_id, e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    eprintln!("[PUT-MSG] Message saved to database: {}", message_id);
    tracing::info!("[MessageSync] Message persisted: {}", message_id);
    
    // Broadcast to subscribers
    broadcast_state.broadcast(conversation_id, chat_message);
    
    Ok(StatusCode::OK)
}
```

---

## 6. Authentication Handlers Changes

### File: `src/backend/auth/handlers/login.rs`

**Pattern for all auth handlers**:
- Remove `Option<PgPool>` parameter checks
- Make user lookup mandatory
- Return proper errors instead of defaults

**Before**:
```rust
pub async fn login(
    State(db_pool): State<Option<PgPool>>,
    // ...
) -> impl IntoResponse {
    if let Some(pool) = db_pool {
        // Check user
    } else {
        return "Database unavailable".into_response();
    }
}
```

**After**:
```rust
pub async fn login(
    State(db_pool): State<PgPool>,
    // ...
) -> Result<Json<LoginResponse>, StatusCode> {
    let user = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE email = $1"
    )
    .bind(&email)
    .fetch_optional(&db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Database error during login: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?
    .ok_or(StatusCode::UNAUTHORIZED)?;
    
    // Verify password
    // Return token
}
```

---

## 7. Middleware Changes

### File: `src/backend/middleware/auth.rs`

**Current Pattern**:
```rust
pub async fn extract_user_id(
    State(db_pool): State<Option<PgPool>>,
    headers: &HeaderMap,
) -> Result<Uuid, StatusCode> {
    let token = extract_token(headers)?;
    
    if let Some(pool) = db_pool {
        verify_token(&pool, token).await
    } else {
        Err(StatusCode::INTERNAL_SERVER_ERROR)
    }
}
```

**New Pattern**:
```rust
pub async fn extract_user_id(
    State(db_pool): State<PgPool>,
    headers: &HeaderMap,
) -> Result<Uuid, StatusCode> {
    let token = extract_token(headers)?;
    
    verify_token(&db_pool, token)
        .await
        .map_err(|e| {
            tracing::error!("Token verification failed: {:?}", e);
            StatusCode::UNAUTHORIZED
        })
}
```

---

## 8. Database Operations Module Changes

### File: `src/backend/messaging/db.rs`

**All database functions should be simplified**:

**Before**:
```rust
pub async fn create_friend_request(
    pool: &PgPool,
    from_user_id: Uuid,
    to_user_id: Uuid,
    // ...
) -> Result<FriendRequest, sqlx::Error> {
    sqlx::query("INSERT INTO ...").execute(pool).await?;
    Ok(FriendRequest { /* ... */ })
}
```

**After** (same, but now these are REQUIRED):
```rust
pub async fn create_friend_request(
    pool: &PgPool,
    from_user_id: Uuid,
    to_user_id: Uuid,
    // ...
) -> Result<FriendRequest, sqlx::Error> {
    // Must succeed - no fallback
    sqlx::query("INSERT INTO ...")
        .execute(pool)
        .await?;
    
    Ok(FriendRequest { /* ... */ })
}
```

---

## 9. Database Migration Strategy

### New Migration File: `migrations/20240105000000_postgresql_exclusive_schema.sql`

```sql
-- PostgreSQL Exclusive Schema Migration
-- Consolidates all tables and adds production-ready constraints

-- Step 1: Ensure all extensions are enabled
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Step 2: Ensure users table has all required columns
ALTER TABLE users
  ADD COLUMN IF NOT EXISTS username VARCHAR(255),
  ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();

CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_users_created_at ON users(created_at DESC);

-- Step 3: Consolidate conversations
CREATE TABLE IF NOT EXISTS conversations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255),
    description TEXT,
    is_direct_message BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_conversations_created_by ON conversations(created_by);
CREATE INDEX IF NOT EXISTS idx_conversations_created_at ON conversations(created_at DESC);

-- Step 4: Conversation participants
CREATE TABLE IF NOT EXISTS conversation_participants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(conversation_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_conversation_participants_user 
  ON conversation_participants(user_id);

-- Step 5: Consolidate messages under chat_messages
ALTER TABLE chat_messages
  ADD COLUMN IF NOT EXISTS conversation_id UUID REFERENCES conversations(id) ON DELETE CASCADE,
  ADD COLUMN IF NOT EXISTS is_read BOOLEAN DEFAULT FALSE,
  ADD COLUMN IF NOT EXISTS is_delivered BOOLEAN DEFAULT TRUE;

ALTER TABLE chat_messages
  ADD CONSTRAINT fk_chat_messages_conversation 
  FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE;

ALTER TABLE chat_messages
  ADD CONSTRAINT fk_chat_messages_sender 
  FOREIGN KEY (sender_id) REFERENCES users(id) ON DELETE CASCADE;

CREATE INDEX IF NOT EXISTS idx_chat_messages_conversation_id 
  ON chat_messages(conversation_id);
CREATE INDEX IF NOT EXISTS idx_chat_messages_sender_id 
  ON chat_messages(sender_id);
CREATE INDEX IF NOT EXISTS idx_chat_messages_created_at 
  ON chat_messages(created_at DESC);

-- Step 6: Friend requests with proper schema
CREATE TABLE IF NOT EXISTS friend_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    from_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    to_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    from_username VARCHAR(255) NOT NULL,
    from_email VARCHAR(255) NOT NULL,
    to_email VARCHAR(255) NOT NULL,
    message TEXT,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    responded_at TIMESTAMPTZ,
    CONSTRAINT different_users CHECK (from_user_id != to_user_id)
);

CREATE INDEX IF NOT EXISTS idx_friend_requests_to_user 
  ON friend_requests(to_user_id);
CREATE INDEX IF NOT EXISTS idx_friend_requests_status 
  ON friend_requests(status);

-- Step 7: Usage tracking with constraints
ALTER TABLE usage_tracking
  ADD CONSTRAINT fk_usage_tracking_user 
  FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE;

CREATE INDEX IF NOT EXISTS idx_usage_tracking_user_id 
  ON usage_tracking(user_id);
CREATE INDEX IF NOT EXISTS idx_usage_tracking_period 
  ON usage_tracking(period_start, period_end);

-- Step 8: Drop deprecated tables (after data migration if needed)
-- DROP TABLE IF EXISTS messages CASCADE;
-- DROP TABLE IF EXISTS version_history CASCADE;

-- Add comments for documentation
COMMENT ON TABLE conversations IS 'Stores conversation metadata and grouping information';
COMMENT ON TABLE chat_messages IS 'Stores all messages in the system with full Braid CRDT support';
COMMENT ON TABLE friend_requests IS 'Manages friend request workflow';
COMMENT ON TABLE usage_tracking IS 'Tracks user activity and usage metrics';
```

---

## 10. Testing Strategy

### Unit Test Template

**File**: `tests/database/postgres_required.rs`

```rust
#[tokio::test]
async fn test_database_required_on_startup() {
    // Ensure server fails to start without DATABASE_URL
    let result = tokio::task::spawn_blocking(|| {
        std::env::remove_var("DATABASE_URL");
    });
    // Expect startup to fail
}

#[tokio::test]
async fn test_database_connection_pool() {
    let db_pool = load_database()
        .await
        .expect("Database should load successfully");
    
    // Verify pool is working
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users")
        .fetch_one(&db_pool)
        .await
        .expect("Query should succeed");
    
    assert!(count.0 >= 0);
}

#[tokio::test]
async fn test_message_persistence() {
    let db_pool = get_test_db().await;
    
    // Create conversation
    let conv_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    
    // Send message - should persist
    let result = save_message_to_db(&db_pool, &message).await;
    assert!(result.is_ok(), "Message should persist");
    
    // Verify message in database
    let loaded = get_messages_for_conversation(&db_pool, conv_id, 10, 0)
        .await
        .expect("Should load messages");
    
    assert!(loaded.len() > 0);
}
```

---

## 11. Error Handling Patterns

### Pattern 1: Required Database Operation

```rust
async fn operation(pool: &PgPool) -> Result<T, StatusCode> {
    sqlx::query_as::<_, T>("SELECT ... FROM ...")
        .fetch_one(pool)
        .await
        .map_err(|e| {
            tracing::error!("Database error: {:?}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })
}
```

### Pattern 2: Handler with Database

```rust
async fn handler(
    State(db_pool): State<PgPool>,
    // ... other params ...
) -> Result<impl IntoResponse, StatusCode> {
    // Operation fails => handler returns error
    let result = operation(&db_pool).await?;
    
    Ok(Json(result))
}
```

### Pattern 3: Batch Operations with Transaction

```rust
async fn batch_operation(
    pool: &PgPool,
    items: Vec<Item>,
) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;
    
    for item in items {
        sqlx::query("INSERT INTO ... VALUES (...)")
            .bind(&item)
            .execute(&mut *tx)
            .await?;
    }
    
    tx.commit().await?;
    Ok(())
}
```

---

## Implementation Checklist

- [ ] Update `src/backend/server/config.rs` - Make database required
- [ ] Update `src/backend/server/state.rs` - Change `Option<PgPool>` to `PgPool`
- [ ] Update `src/backend/main.rs` - Fail on database unavailability
- [ ] Update `src/backend/messaging/message_sync.rs` - Remove Option guards
- [ ] Update `src/backend/auth/handlers/*.rs` - Make DB mandatory
- [ ] Update `src/backend/chat/handlers/*.rs` - Make DB mandatory
- [ ] Update `src/backend/middleware/auth.rs` - Use required pool
- [ ] Create migration `20240105000000_postgresql_exclusive_schema.sql`
- [ ] Update all database function signatures in `db.rs` files
- [ ] Add database startup validation
- [ ] Create integration tests
- [ ] Update documentation
- [ ] Test full end-to-end flow

---

## Common Pitfalls to Avoid

1. **Partial Updates**: Don't update handlers without updating state extraction
2. **Error Swallowing**: Always propagate database errors to handlers
3. **Missing Constraints**: Ensure foreign keys are created
4. **Migration Order**: Run migrations for constraints after tables exist
5. **Backup Data**: Export data from optional schema before dropping tables
6. **Connection Pooling**: Ensure connection pool settings are appropriate for production
7. **Logging**: Add comprehensive logging to all database operations

