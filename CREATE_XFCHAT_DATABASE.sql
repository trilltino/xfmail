-- XFChat PostgreSQL Database Setup
-- Run this file to create the xfchat database
-- Command: psql -U postgres -f CREATE_XFCHAT_DATABASE.sql

-- This will silently fail if database already exists (which is fine)
CREATE DATABASE xfchat;

-- Connect to the xfchat database
\c xfchat

-- Create extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Create user for xfchat (optional - use default postgres user if preferred)
-- CREATE USER xfchat_user WITH PASSWORD 'Ab13cba46def79_';
-- GRANT ALL PRIVILEGES ON DATABASE xfchat TO xfchat_user;

-- Alternative: Use default postgres user with xfchat database
-- Connection string example:
-- postgresql://postgres:Ab13cba46def79_@localhost:5432/xfchat

-- Set default connection settings for xfchat
ALTER DATABASE xfchat SET timezone TO 'UTC';
ALTER DATABASE xfchat SET default_with_oids = false;