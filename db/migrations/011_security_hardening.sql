-- ============================================================
-- Migration 011: Security Hardening
-- ============================================================
-- 1. Hash refresh tokens in storage (SHA-256 hex, 64 chars)
-- 2. Add index for hash-based lookups
-- NOTE: Existing plaintext tokens become invalid on deploy.
--       All users must re-login after applying this migration.
-- ============================================================

-- Add token_hash column (SHA-256 hex of the JWT refresh token)
ALTER TABLE user_sessions
    ADD COLUMN IF NOT EXISTS token_hash VARCHAR(64),
    ALTER COLUMN refresh_token DROP NOT NULL;

-- Index for fast hash lookups during refresh/logout
CREATE INDEX IF NOT EXISTS idx_user_sessions_token_hash
    ON user_sessions(token_hash)
    WHERE token_hash IS NOT NULL;

-- Invalidate all existing plaintext-stored sessions so users
-- re-authenticate with the new hashed flow.
UPDATE user_sessions SET revoked = TRUE WHERE token_hash IS NULL;

-- ── Audit Log Table ──────────────────────────────────────────
-- Placed here as it supports security audit requirements.
CREATE TABLE IF NOT EXISTS audit_logs (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    org_id          UUID REFERENCES organizations(id) ON DELETE SET NULL,
    user_id         UUID REFERENCES users(id) ON DELETE SET NULL,
    action          VARCHAR(100) NOT NULL,       -- e.g. 'login', 'campaign.launched'
    resource_type   VARCHAR(100),                -- e.g. 'campaign', 'contact'
    resource_id     UUID,
    ip_address      INET,
    user_agent      TEXT,
    metadata        JSONB NOT NULL DEFAULT '{}', -- extra context
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_logs_org_id    ON audit_logs(org_id);
CREATE INDEX idx_audit_logs_user_id   ON audit_logs(user_id);
CREATE INDEX idx_audit_logs_action    ON audit_logs(action);
CREATE INDEX idx_audit_logs_created   ON audit_logs(created_at DESC);
CREATE INDEX idx_audit_logs_resource  ON audit_logs(resource_type, resource_id)
    WHERE resource_type IS NOT NULL;
