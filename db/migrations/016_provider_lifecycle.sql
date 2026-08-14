-- ============================================================
-- Migration 016: Provider abstraction + message lifecycle
-- ============================================================

ALTER TYPE message_status ADD VALUE IF NOT EXISTS 'processing';
ALTER TYPE message_status ADD VALUE IF NOT EXISTS 'cancelled';
ALTER TYPE message_status ADD VALUE IF NOT EXISTS 'retrying';

UPDATE messages SET status = 'processing' WHERE status = 'sending';

ALTER TABLE whatsapp_accounts
    ADD COLUMN IF NOT EXISTS provider VARCHAR(32) NOT NULL DEFAULT 'mock',
    ADD COLUMN IF NOT EXISTS last_health_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS last_health_status VARCHAR(32),
    ADD COLUMN IF NOT EXISTS last_sync_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS environment VARCHAR(32) NOT NULL DEFAULT 'development';

DELETE FROM conversations a
USING conversations b
WHERE a.ctid < b.ctid
  AND a.organization_id = b.organization_id
  AND a.contact_id = b.contact_id
  AND a.wa_account_id IS NOT DISTINCT FROM b.wa_account_id;

CREATE UNIQUE INDEX IF NOT EXISTS idx_conversations_org_contact_account
    ON conversations (organization_id, contact_id, COALESCE(wa_account_id, '00000000-0000-0000-0000-000000000000'));

CREATE TABLE IF NOT EXISTS message_status_events (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    message_id      UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    from_status     VARCHAR(32),
    to_status       VARCHAR(32) NOT NULL,
    reason          TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_message_status_events_message ON message_status_events(message_id);

CREATE TABLE IF NOT EXISTS quick_replies (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    title           VARCHAR(120) NOT NULL,
    body            TEXT NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_quick_replies_org ON quick_replies(organization_id);
