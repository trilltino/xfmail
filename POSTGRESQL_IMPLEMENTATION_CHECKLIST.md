# PostgreSQL Exclusive Implementation Checklist (XFChat)

Quick reference for tracking implementation progress for XFChat project. Mark items as complete with `[x]`.

## Phase 1: Pre-Implementation Setup

- [ ] Review `POSTGRESQL_MIGRATION_PLAN.md` with team
- [ ] Create feature branch: `feature/postgresql-exclusive`
- [ ] Backup current database
- [ ] Create staging environment for testing
- [ ] Review current `Option<PgPool>` usage patterns
- [ ] Document any custom database configurations

## Phase 2: Database Schema Migration

### Schema Files
- [ ] Create migration `migrations/20240105000000_postgresql_exclusive_schema.sql`
  - [ ] Add conversations table
  - [ ] Add conversation_participants table
  - [ ] Add foreign key constraints
  - [ ] Create performance indexes
  - [ ] Add CHECK constraints
  
### Verification
- [ ] Run migrations locally: `sqlx migrate run`
- [ ] Verify all tables created: `\dt` in psql
- [ ] Verify all indexes: `\di` in psql
- [ ] Verify constraints: `\d table_name` for each table

### Data Migration (if needed)
- [ ] Export data from deprecated tables
- [ ] Migrate data to new schema
- [ ] Verify data integrity
- [ ] Update foreign key relationships

## Phase 3: Server Configuration

### `src/backend/server/config.rs`
- [ ] Change `DatabaseConfig` type: `Option<PgPool>` → `PgPool`
- [ ] Update `load_database()` return type to `Result<PgPool, Box<dyn Error>>`
- [ ] Remove graceful degradation for missing DATABASE_URL
- [ ] Remove graceful degradation for connection failures
- [ ] Make migrations mandatory (fail if they don't run)
- [ ] Add test connection verification
- [ ] Update error messages to be clear and actionable

### `src/backend/server/state.rs`
- [ ] Change `AppState::db_pool` field: `Option<PgPool>` → `PgPool`
- [ ] Update `FromRef<AppState> for Option<PgPool>` to `FromRef<AppState> for PgPool`
- [ ] Update struct documentation
- [ ] Remove commentary about optional database

### `src/backend/main.rs`
- [ ] Call `load_database().await` and handle Result
- [ ] Fail fast if database connection unavailable
- [ ] Add database health check before binding server
- [ ] Update startup messages with `[STARTUP]` tags
- [ ] Print database connection status
- [ ] Print user count or other verification
- [ ] Remove database availability warnings

## Phase 4: Handler Updates

### Message Subscription Handler
**File**: `src/backend/messaging/message_sync.rs`
- [ ] Change parameter: `State(Option<PgPool>)` → `State(PgPool)`
- [ ] Remove `if let Some(pool)` guards
- [ ] Make participant check mandatory
- [ ] Make message loading mandatory with error propagation
- [ ] Update logging tags to be consistent

### Message PUT Handler
**File**: `src/backend/messaging/message_sync.rs`
- [ ] Change parameter: `State(Option<PgPool>)` → `State(PgPool>`
- [ ] Remove `if let Some(pool)` guards  
- [ ] Make message persistence mandatory
- [ ] Return error if persistence fails
- [ ] Update logging to show persistence status

### Authentication Handlers
**Files**: `src/backend/auth/handlers/login.rs`, `signup.rs`, `me.rs`
- [ ] `login.rs`: Change `State(Option<PgPool>)` → `State(PgPool)`
  - [ ] Remove optional database checks
  - [ ] Make user lookup mandatory
  - [ ] Return proper error responses
- [ ] `signup.rs`: Change `State(Option<PgPool>)` → `State(PgPool)`
  - [ ] Make user creation mandatory
  - [ ] Add proper constraint violation handling
- [ ] `me.rs`: Change `State(Option<PgPool>)` → `State(PgPool)`
  - [ ] Make user info lookup mandatory

### Chat Handlers
**File**: `src/backend/chat/handlers/put.rs`
- [ ] Change parameter: `State(Option<PgPool>)` → `State(PgPool>`
- [ ] Remove optional database checks
- [ ] Ensure all messages persisted
- [ ] Proper error responses

### Other Handlers
**File**: `src/backend/middleware/auth.rs`
- [ ] Update token verification to use required pool
- [ ] Remove optional checks
- [ ] Proper error handling

## Phase 5: Database Layer Updates

### Messaging Database Module
**File**: `src/backend/messaging/db.rs`
- [ ] Review all function signatures
- [ ] Verify all take `&PgPool` (required)
- [ ] Check error handling is appropriate
- [ ] Add logging for debugging
- [ ] Document function purposes

### Chat Database Module
**File**: `src/backend/chat/db.rs`
- [ ] Review all function signatures
- [ ] Verify all take `&PgPool` (required)
- [ ] Check error handling is appropriate
- [ ] Verify migrations compatibility
- [ ] Test with actual database

### User Management
**File**: `src/backend/auth/users.rs`
- [ ] Update user queries to use required pool
- [ ] Remove any optional checks
- [ ] Add comprehensive error handling
- [ ] Document user operations

## Phase 6: Testing

### Unit Tests
- [ ] Create test for missing DATABASE_URL → startup fails
- [ ] Create test for connection failure → startup fails
- [ ] Create test for migration failure → startup fails
- [ ] Create test for message persistence
- [ ] Create test for user authentication
- [ ] Create test for conversation management
- [ ] Create test for friend requests

### Integration Tests
- [ ] Test end-to-end user registration
- [ ] Test end-to-end message flow
- [ ] Test concurrent message writes
- [ ] Test database connection pooling
- [ ] Test transaction rollback on error
- [ ] Test concurrent subscription handling
- [ ] Test friend request workflow

### Startup Validation Tests
- [ ] Verify server fails without DATABASE_URL
- [ ] Verify server fails if PostgreSQL is down
- [ ] Verify server fails if migrations fail
- [ ] Verify health check endpoint works

### Load Tests
- [ ] Test with 100 concurrent users
- [ ] Test with 1000 messages per conversation
- [ ] Test connection pool exhaustion handling
- [ ] Measure query performance

## Phase 7: Documentation & Production Readiness

### Code Documentation
- [ ] Add doc comments explaining required database
- [ ] Update README with setup instructions
- [ ] Document environment variables
- [ ] Document error codes and responses
- [ ] Add troubleshooting guide

### Operational Documentation
- [ ] Database backup procedure
- [ ] Database recovery procedure
- [ ] Migration rollback procedure
- [ ] Performance monitoring guide
- [ ] Connection pool tuning guide

### Configuration Documentation
- [ ] Document all environment variables
- [ ] Document connection string format
- [ ] Document SSL/TLS setup
- [ ] Document connection pool settings

### Production Checklist
- [ ] [ ] DATABASE_URL configured and verified
- [ ] [ ] Connection pooling optimized (25-50 connections)
- [ ] [ ] Query timeout set appropriate (30 seconds)
- [ ] [ ] SSL/TLS enabled for database connection
- [ ] [ ] Database backups scheduled
- [ ] [ ] Database monitoring enabled
- [ ] [ ] Query logging configured
- [ ] [ ] Slow query alerting configured
- [ ] [ ] Database migration tested on staging
- [ ] [ ] Rollback plan tested
- [ ] [ ] Load testing completed
- [ ] [ ] Error scenarios tested
- [ ] [ ] Health check endpoint verified
- [ ] [ ] Logging shows database status on startup
- [ ] [ ] Documentation reviewed and signed off

## Phase 8: Deployment

### Pre-Deployment
- [ ] All tests passing
- [ ] Code reviewed and approved
- [ ] Staging environment tests successful
- [ ] Documentation complete
- [ ] Team trained on new architecture

### Deployment Steps
- [ ] Tag release version
- [ ] Build release artifacts
- [ ] Deploy to staging for final verification
- [ ] Deploy to production
- [ ] Monitor logs for errors
- [ ] Verify health checks passing
- [ ] Confirm message delivery working
- [ ] Confirm user authentication working

### Post-Deployment
- [ ] Monitor database performance
- [ ] Monitor connection pool usage
- [ ] Monitor error rates
- [ ] Run smoke tests
- [ ] Verify backup procedures working
- [ ] Document any issues encountered
- [ ] Update incident response procedures

## Rollback Criteria

Rollback if any of these occur:
- [ ] Server fails to start in production
- [ ] Database connections consistently failing
- [ ] Query performance degrades by >50%
- [ ] More than 5% of requests returning 500 errors
- [ ] Message delivery failures
- [ ] User authentication failures
- [ ] Critical data corruption detected

## Success Metrics

- [ ] All handlers use required `PgPool` (not `Option<PgPool>`)
- [ ] All database operations return errors (not fallback)
- [ ] Server startup fails fast if database unavailable
- [ ] All tests passing
- [ ] Database schema unified (single source of truth)
- [ ] Performance indexes in place
- [ ] Foreign key constraints enforced
- [ ] Production deployment successful
- [ ] Zero message loss in migration
- [ ] User experience unchanged

## Post-Implementation Tasks

- [ ] Remove old database optional code comments
- [ ] Remove old environment variable docs for optional database
- [ ] Update architecture diagrams
- [ ] Update deployment procedures
- [ ] Schedule team training session
- [ ] Add database administration guide to wiki
- [ ] Update on-call procedures
- [ ] Archive old database-optional code branch

## Quick Reference: Files to Modify

Essential changes needed:
```
src/backend/server/
  ├── config.rs (make database required)
  ├── state.rs (change Option<PgPool> to PgPool)
  └── main.rs (add database startup validation)

src/backend/messaging/
  ├── message_sync.rs (remove Option checks)
  ├── db.rs (ensure required pool usage)
  └── handlers.rs (update signatures)

src/backend/auth/
  ├── handlers/login.rs (remove Option checks)
  ├── handlers/signup.rs (remove Option checks)
  ├── handlers/me.rs (remove Option checks)
  └── users.rs (ensure required pool)

src/backend/chat/
  ├── db.rs (verify required pool)
  └── handlers/put.rs (remove Option checks)

src/backend/middleware/
  └── auth.rs (update token verification)

migrations/
  └── 20240105000000_postgresql_exclusive_schema.sql (new)

tests/
  ├── database/postgres_required.rs (new)
  └── integration/startup.rs (new)
```

## Estimated Time Per Phase

| Phase | Estimate | Status |
|-------|----------|--------|
| 1: Analysis | 1 hour | ⏳ |
| 2: Schema | 2 hours | ⏳ |
| 3: Config | 3 hours | ⏳ |
| 4: Handlers | 6 hours | ⏳ |
| 5: Database Layer | 3 hours | ⏳ |
| 6: Testing | 5 hours | ⏳ |
| 7: Documentation | 3 hours | ⏳ |
| 8: Deployment | 2 hours | ⏳ |
| **Total** | **25 hours** | |

## Notes

- Keep database-optional commit history for reference
- Test each phase independently before moving to next
- Maintain staging database for testing
- Document any deviations from plan
- Brief team on changes before deployment

