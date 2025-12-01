-- Add missing columns to friend_requests table
ALTER TABLE friend_requests ADD COLUMN IF NOT EXISTS from_username VARCHAR(30);
ALTER TABLE friend_requests ADD COLUMN IF NOT EXISTS from_email VARCHAR(255);
ALTER TABLE friend_requests ADD COLUMN IF NOT EXISTS to_email VARCHAR(255);
ALTER TABLE friend_requests ADD COLUMN IF NOT EXISTS responded_at TIMESTAMPTZ;

-- Add missing columns to contacts table
ALTER TABLE contacts ADD COLUMN IF NOT EXISTS username VARCHAR(30);
ALTER TABLE contacts ADD COLUMN IF NOT EXISTS email VARCHAR(255);
ALTER TABLE contacts ADD COLUMN IF NOT EXISTS avatar_url VARCHAR(255);
ALTER TABLE contacts ADD COLUMN IF NOT EXISTS last_seen TIMESTAMPTZ;
ALTER TABLE contacts ADD COLUMN IF NOT EXISTS is_online BOOLEAN DEFAULT false;

-- Add missing columns to messages table
ALTER TABLE messages ADD COLUMN IF NOT EXISTS is_read BOOLEAN DEFAULT FALSE;
ALTER TABLE messages ADD COLUMN IF NOT EXISTS is_delivered BOOLEAN DEFAULT FALSE;
