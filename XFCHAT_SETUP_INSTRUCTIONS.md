# XFChat PostgreSQL Database Setup Instructions

## Overview
This document provides step-by-step instructions to set up the XFChat PostgreSQL database with the password `Ab13cba46def79_`.

## Prerequisites
- PostgreSQL 12+ installed and running
- Access to PostgreSQL as the `postgres` user
- Basic knowledge of command line operations

## Step 1: Create XFChat Database

### Method 1: Using SQL File
```bash
# Connect to PostgreSQL as postgres user
psql -U postgres

# Then run the database creation script
\i CREATE_XFCHAT_DATABASE.sql
```

### Method 2: Direct Commands
```bash
# Create the database directly
createdb -U postgres xfchat

# Or using psql
psql -U postgres -c "CREATE DATABASE xfchat;"
```

## Step 2: Set Database Password

### For Default postgres User
```bash
# Connect to PostgreSQL and set password
psql -U postgres -c "ALTER USER postgres PASSWORD 'Ab13cba46def79_';"
```

### For Custom User (Optional)
```bash
# Create a dedicated XFChat user
psql -U postgres -c "CREATE USER xfchat_user WITH PASSWORD 'Ab13cba46def79_';"
psql -U postgres -c "GRANT ALL PRIVILEGES ON DATABASE xfchat TO xfchat_user;"
```

## Step 3: Apply Database Schema

### Using Migration File
```bash
# Navigate to the project directory
cd /path/to/xfchat/project

# Set environment variables
export DATABASE_URL="postgresql://postgres:Ab13cba46def79_@localhost:5432/xfchat"

# Run migrations
sqlx migrate run

# Or if using the migration file directly:
psql -U postgres -d xfchat -f migrations/20240105000000_xfchat_comprehensive_schema.sql
```

### Using SQL File Directly
```bash
# Apply the comprehensive schema directly
psql -U postgres -d xfchat -f migrations/20240105000000_xfchat_comprehensive_schema.sql
```

## Step 4: Verify Setup

### Test Connection
```bash
# Test database connection
psql -U postgres -d xfchat -c "SELECT current_database(), current_user, version();"

# Expected output should show:
# - Database: xfchat
# - User: postgres
# - PostgreSQL version
```

### Verify Tables
```bash
# List all tables in the database
psql -U postgres -d xfchat -c "\dt"

# Expected tables:
# - users
# - conversations  
# - conversation_participants
# - chat_messages
# - friend_requests
# - contacts
# - usage_tracking
# - messages (legacy)
# - version_history (legacy)
```

### Test Extensions
```bash
# Verify required extensions are installed
psql -U postgres -d xfchat -c "SELECT extname FROM pg_extension WHERE extname IN ('uuid-ossp', 'pgcrypto');"

# Expected output:
# extname
#----------
# pgcrypto
# uuid-ossp
#(2 rows)
```

## Step 5: Configure Application

### Environment Variables
Create or update your `.env` file:
```bash
# XFChat Database Configuration
DATABASE_URL=postgresql://postgres:Ab13cba46def79_@localhost:5432/xfchat

# Alternative: Using custom user
# DATABASE_URL=postgresql://xfchat_user:Ab13cba46def79_@localhost:5432/xfchat

# Server Configuration
SERVER_PORT=3000
RUST_LOG=info,xfchat=debug

# Development Settings
DEV_AUTH_BYPASS=0
```

### For Production
```bash
# Set environment variables for production
export DATABASE_URL="postgresql://postgres:Ab13cba46def79_@db-server:5432/xfchat"
export RUST_LOG="warn,xfchat=error"
export DEV_AUTH_BYPASS="0"
```

## Troubleshooting

### Connection Issues
```bash
# Check if PostgreSQL is running
sudo systemctl status postgresql

# Check if PostgreSQL is listening on port 5432
sudo netstat -tlnp | grep 5432

# Test connection with verbose output
psql -U postgres -h localhost -d xfchat -v
```

### Permission Issues
```bash
# Grant permissions to the user
psql -U postgres -c "GRANT ALL PRIVILEGES ON DATABASE xfchat TO postgres;"
psql -U postgres -d xfchat -c "GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO postgres;"
psql -U postgres -d xfchat -c "GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO postgres;"
```

### Reset Database (if needed)
```bash
# Drop and recreate database
psql -U postgres -c "DROP DATABASE IF EXISTS xfchat;"
psql -U postgres -c "CREATE DATABASE xfchat;"

# Then re-run the migration
psql -U postgres -d xfchat -f migrations/20240105000000_xfchat_comprehensive_schema.sql
```

## Database Schema Overview

The XFChat database includes:

### Core Tables
- **users**: User accounts and authentication
- **conversations**: Chat conversation metadata
- **conversation_participants**: User membership in conversations
- **chat_messages**: Messages with Braid CRDT support
- **friend_requests**: Friend request workflow
- **contacts**: User contact list and presence

### Legacy Tables (for compatibility)
- **messages**: Legacy message storage
- **version_history**: Legacy CRDT version tracking

### Utility Tables
- **usage_tracking**: User activity and usage metrics

### Features
- Full foreign key constraints
- Automated timestamp triggers
- Performance indexes
- Braid CRDT support for synchronization
- UUID primary keys with proper extensions

## Support

If you encounter issues:
1. Check PostgreSQL logs: `sudo tail -f /var/log/postgresql/postgresql-*.log`
2. Verify network connectivity to database server
3. Ensure database password is correct
4. Check file permissions for migration scripts
5. Verify PostgreSQL user has sufficient privileges

## Security Notes

- Change the default password in production environments
- Use SSL/TLS for database connections in production
- Restrict database user permissions to minimum required
- Regularly backup the database
- Monitor database access logs

---

**Database**: `xfchat`  
**Password**: `Ab13cba46def79_`  
**Port**: `5432` (default PostgreSQL port)  
**Extensions**: `uuid-ossp`, `pgcrypto`