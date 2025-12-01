# XFMail Braid Sync Testing Guide

## Status
✅ **Code fixes complete**. Ready for testing once Docker is running.

## Prerequisites
- Docker Desktop must be running
- PostgreSQL container from `docker-compose.yml` must be active

## Fixed Issues
1. **Migration table mismatch**: Created `chat_messages` table (was `messages`)
2. **Missing columns**: Added `is_read`, `is_delivered` to message schema
3. **Graceful fallback**: Init code no longer crashes when old tables don't exist
4. **Environment variables**: Client now reads `CLIENT_API_URL` and `DEV_USER_ID`

## To Start Testing

### Step 1: Start Docker & Database
```bash
docker-compose down
docker-compose up -d
# Wait 10 seconds for PostgreSQL to be ready
```

### Step 2: Run Full Test Suite
```bash
start_test.bat
```

This will:
- Build backend server with SSR feature
- Build egui client application
- Launch backend on `127.0.0.1:3000`
- Launch Client 1 (User: `00000000-0000-0000-0000-000000000001`)
- Launch Client 2 (User: `00000000-0000-0000-0000-000000000002`)

### Step 3: Test Messaging
1. Login to both clients with `alice`/`password`
2. Create a conversation in Client 1
3. Add Client 2's user as participant
4. Send messages from either client

### Expected Output
- **Server console**: `[GET-SUB]` tags when clients subscribe
- **Server console**: `[PUT-MSG]` tags when messages arrive
- **Client consoles**: `[CLIENT-SUB]` tags showing subscription status
- Messages appear in real-time via Braid-HTTP SSE stream

## Key Debug Tags
- `[STARTUP]` - Server initialization
- `[GET-SUB]` - Subscription requests received by server
- `[PUT-MSG]` - Message PUT requests received by server
- `[CLIENT-SUB]` - Client subscription attempts and status

## New Migrations
- `20240102000000_add_username_and_messaging.sql` - Messaging tables (updated with `chat_messages`)
- `20240104000000_braid_sync_messages.sql` - Ensures Braid sync tables exist

## Troubleshooting

### "chat_messages does not exist"
- Docker/database not running
- Migrations haven't executed
- Solution: Run `docker-compose up -d` and wait 10 seconds

### Client won't connect
- Check `DEV_AUTH_BYPASS=1` is set in start_test.bat
- Check server is running on port 3000
- Check `RUST_LOG=debug` for detailed output

### Subscription fails with 500 error
- Database connection failed
- Solution: Ensure Docker container is healthy with `docker ps`

## Files Modified
- `/migrations/20240102000000_add_username_and_messaging.sql` - Fixed table name
- `/migrations/20240104000000_braid_sync_messages.sql` - New migration
- `/src/backend/server/init.rs` - Graceful error handling
- `/src/egui_app/config.rs` - Environment variable support
- `/start_test.bat` - Test harness
