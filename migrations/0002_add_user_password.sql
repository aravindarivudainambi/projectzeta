-- 0002_add_user_password.sql
-- Ensures users table stores credential hashes.

ALTER TABLE users
ADD COLUMN IF NOT EXISTS password TEXT NOT NULL DEFAULT '';
