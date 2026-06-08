-- ============================================================
-- Migration 001: Users & Authentication
-- ============================================================

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Enum types
CREATE TYPE user_role AS ENUM ('super_admin', 'business_admin', 'team_member');
CREATE TYPE user_status AS ENUM ('active', 'inactive', 'suspended', 'pending_verification');

CREATE TABLE IF NOT EXISTS users (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email           VARCHAR(255) NOT NULL UNIQUE,
    password_hash   VARCHAR(255) NOT NULL,
    first_name      VARCHAR(100) NOT NULL,
    last_name       VARCHAR(100) NOT NULL,
    avatar_url      TEXT,
    role            user_role NOT NULL DEFAULT 'team_member',
    status          user_status NOT NULL DEFAULT 'pending_verification',

    -- Email verification
    email_verified          BOOLEAN NOT NULL DEFAULT FALSE,
    email_verify_token      VARCHAR(255),
    email_verify_expires_at TIMESTAMPTZ,

    -- 2FA
    two_factor_enabled      BOOLEAN NOT NULL DEFAULT FALSE,
    two_factor_secret       VARCHAR(255),           -- TOTP secret (encrypted)
    two_factor_backup_codes TEXT[],                 -- encrypted backup codes

    -- Password reset
    password_reset_token        VARCHAR(255),
    password_reset_expires_at   TIMESTAMPTZ,

    -- Refresh tokens (stored in Redis, but track invalidation here)
    token_version   INTEGER NOT NULL DEFAULT 0,     -- bump to invalidate all tokens

    -- Metadata
    last_login_at   TIMESTAMPTZ,
    last_login_ip   INET,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at      TIMESTAMPTZ                     -- soft delete
);

CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_status ON users(status);
CREATE INDEX idx_users_role ON users(role);
CREATE INDEX idx_users_deleted_at ON users(deleted_at) WHERE deleted_at IS NULL;

-- ── Refresh token sessions (persistent login tracking) ───────
CREATE TABLE IF NOT EXISTS user_sessions (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    refresh_token   VARCHAR(512) NOT NULL UNIQUE,
    user_agent      TEXT,
    ip_address      INET,
    expires_at      TIMESTAMPTZ NOT NULL,
    revoked         BOOLEAN NOT NULL DEFAULT FALSE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_user_sessions_user_id ON user_sessions(user_id);
CREATE INDEX idx_user_sessions_refresh_token ON user_sessions(refresh_token);

-- ── Auto-update updated_at trigger ──────────────────────────
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
