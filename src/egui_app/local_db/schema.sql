-- Local SQLite Database Schema
-- Mirrors the server PostgreSQL schema for offline functionality

-- Users table (local user data)
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE,
    display_name TEXT,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    -- Sync metadata
    sync_version INTEGER DEFAULT 0,
    last_synced_at TEXT,
    needs_sync BOOLEAN DEFAULT 0
);

-- Contacts table (friends and relationships)
CREATE TABLE IF NOT EXISTS contacts (
    id TEXT PRIMARY KEY,
    contact_user_id TEXT NOT NULL,
    username TEXT NOT NULL,
    email TEXT NOT NULL,
    display_name TEXT,
    status TEXT NOT NULL DEFAULT 'active', -- active, blocked, pending
    friendship_status TEXT NOT NULL DEFAULT 'none', -- none, requested, accepted
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    -- Sync metadata
    sync_version INTEGER DEFAULT 0,
    last_synced_at TEXT,
    needs_sync BOOLEAN DEFAULT 0,
    UNIQUE(contact_user_id)
);

-- Conversations table
CREATE TABLE IF NOT EXISTS conversations (
    id TEXT PRIMARY KEY,
    name TEXT, -- Optional conversation name
    conversation_type TEXT NOT NULL DEFAULT 'direct', -- direct, group
    created_by TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    -- Sync metadata
    sync_version INTEGER DEFAULT 0,
    last_synced_at TEXT,
    needs_sync BOOLEAN DEFAULT 0,
    FOREIGN KEY (created_by) REFERENCES users(id)
);

-- Conversation participants
CREATE TABLE IF NOT EXISTS conversation_participants (
    conversation_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    role TEXT NOT NULL DEFAULT 'member', -- member, admin
    joined_at TEXT NOT NULL,
    -- Sync metadata
    sync_version INTEGER DEFAULT 0,
    last_synced_at TEXT,
    needs_sync BOOLEAN DEFAULT 0,
    PRIMARY KEY (conversation_id, user_id),
    FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id)
);

-- Messages table
CREATE TABLE IF NOT EXISTS messages (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL,
    sender_id TEXT NOT NULL,
    content TEXT NOT NULL,
    message_type TEXT NOT NULL DEFAULT 'text',
    timestamp TEXT NOT NULL,
    is_read BOOLEAN NOT NULL DEFAULT 0,
    is_delivered BOOLEAN NOT NULL DEFAULT 0,
    crdt_timestamp INTEGER NOT NULL DEFAULT 0,
    braid_version TEXT NOT NULL,
    braid_parents TEXT NOT NULL, -- JSON array
    -- Local metadata
    delivery_status TEXT NOT NULL DEFAULT 'sending', -- sending, sent, delivered, failed
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    -- Sync metadata
    sync_version INTEGER DEFAULT 0,
    last_synced_at TEXT,
    needs_sync BOOLEAN DEFAULT 0,
    FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE,
    FOREIGN KEY (sender_id) REFERENCES users(id)
);

-- Friend requests (outgoing and incoming)
CREATE TABLE IF NOT EXISTS friend_requests (
    id TEXT PRIMARY KEY,
    from_user_id TEXT NOT NULL,
    to_user_id TEXT NOT NULL,
    message TEXT,
    status TEXT NOT NULL DEFAULT 'pending', -- pending, accepted, rejected
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    -- Sync metadata
    sync_version INTEGER DEFAULT 0,
    last_synced_at TEXT,
    needs_sync BOOLEAN DEFAULT 0,
    FOREIGN KEY (from_user_id) REFERENCES users(id),
    FOREIGN KEY (to_user_id) REFERENCES users(id),
    UNIQUE(from_user_id, to_user_id)
);

-- Sync metadata table
CREATE TABLE IF NOT EXISTS sync_metadata (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Offline message queue
CREATE TABLE IF NOT EXISTS offline_queue (
    id TEXT PRIMARY KEY,
    operation_type TEXT NOT NULL, -- message_send, friend_request, etc.
    data TEXT NOT NULL, -- JSON payload
    created_at TEXT NOT NULL,
    retry_count INTEGER DEFAULT 0,
    last_attempt TEXT,
    error_message TEXT
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_messages_conversation_timestamp ON messages(conversation_id, timestamp);
CREATE INDEX IF NOT EXISTS idx_messages_sender ON messages(sender_id);
CREATE INDEX IF NOT EXISTS idx_contacts_user ON contacts(contact_user_id);
CREATE INDEX IF NOT EXISTS idx_conversation_participants_conversation ON conversation_participants(conversation_id);
CREATE INDEX IF NOT EXISTS idx_friend_requests_users ON friend_requests(from_user_id, to_user_id);
CREATE INDEX IF NOT EXISTS idx_offline_queue_created ON offline_queue(created_at);

-- Triggers for updated_at
CREATE TRIGGER IF NOT EXISTS update_users_updated_at
    AFTER UPDATE ON users
    FOR EACH ROW
    WHEN NEW.updated_at != OLD.updated_at
    BEGIN
        UPDATE users SET updated_at = datetime('now') WHERE id = NEW.id;
    END;

CREATE TRIGGER IF NOT EXISTS update_contacts_updated_at
    AFTER UPDATE ON contacts
    FOR EACH ROW
    WHEN NEW.updated_at != OLD.updated_at
    BEGIN
        UPDATE contacts SET updated_at = datetime('now') WHERE id = NEW.id;
    END;

CREATE TRIGGER IF NOT EXISTS update_conversations_updated_at
    AFTER UPDATE ON conversations
    FOR EACH ROW
    WHEN NEW.updated_at != OLD.updated_at
    BEGIN
        UPDATE conversations SET updated_at = datetime('now') WHERE id = NEW.id;
    END;

CREATE TRIGGER IF NOT EXISTS update_messages_updated_at
    AFTER UPDATE ON messages
    FOR EACH ROW
    WHEN NEW.updated_at != OLD.updated_at
    BEGIN
        UPDATE messages SET updated_at = datetime('now') WHERE id = NEW.id;
    END;