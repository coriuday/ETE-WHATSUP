-- ============================================================
-- Migration 006: Messages (Individual Delivery Records)
-- ============================================================

CREATE TYPE message_status AS ENUM (
    'queued',
    'sending',
    'sent',
    'delivered',
    'read',
    'failed',
    'rejected'
);

CREATE TYPE message_direction AS ENUM ('outbound', 'inbound');

CREATE TYPE message_type AS ENUM (
    'text',
    'image',
    'video',
    'audio',
    'document',
    'location',
    'contact',
    'sticker',
    'template',
    'interactive',
    'reaction'
);

-- ── Messages ─────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS messages (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id     UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    wa_account_id       UUID REFERENCES whatsapp_accounts(id) ON DELETE SET NULL,
    campaign_id         UUID REFERENCES campaigns(id) ON DELETE SET NULL,
    contact_id          UUID REFERENCES contacts(id) ON DELETE SET NULL,
    conversation_id     UUID,                           -- FK added in 008

    -- WhatsApp identifiers
    wa_message_id       VARCHAR(255),                   -- Meta's wamid
    wa_context_id       VARCHAR(255),                   -- replied-to wamid

    -- Content
    direction           message_direction NOT NULL DEFAULT 'outbound',
    type                message_type NOT NULL DEFAULT 'text',
    body                TEXT,
    media_url           TEXT,
    media_mime_type     VARCHAR(100),
    media_size          BIGINT,
    media_filename      TEXT,
    template_name       VARCHAR(255),                   -- if template message
    template_variables  JSONB,
    buttons             JSONB,                          -- interactive buttons/lists
    reaction_emoji      VARCHAR(10),                    -- for reaction type

    -- Status tracking
    status              message_status NOT NULL DEFAULT 'queued',
    error_code          VARCHAR(50),
    error_message       TEXT,

    -- Timestamps (WhatsApp delivery pipeline)
    queued_at           TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    sent_at             TIMESTAMPTZ,
    delivered_at        TIMESTAMPTZ,
    read_at             TIMESTAMPTZ,
    failed_at           TIMESTAMPTZ,

    -- Meta
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_messages_org_id ON messages(organization_id);
CREATE INDEX idx_messages_campaign_id ON messages(campaign_id);
CREATE INDEX idx_messages_contact_id ON messages(contact_id);
CREATE INDEX idx_messages_conversation_id ON messages(conversation_id);
CREATE INDEX idx_messages_wa_message_id ON messages(wa_message_id);
CREATE INDEX idx_messages_status ON messages(status);
CREATE INDEX idx_messages_direction ON messages(direction);
CREATE INDEX idx_messages_created_at ON messages(created_at DESC);

CREATE TRIGGER update_messages_updated_at
    BEFORE UPDATE ON messages
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ── Message Queue (Redis-backed, but DB as fallback/audit) ───
CREATE TABLE IF NOT EXISTS message_queue_jobs (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    campaign_id     UUID NOT NULL REFERENCES campaigns(id) ON DELETE CASCADE,
    contact_id      UUID NOT NULL REFERENCES contacts(id) ON DELETE CASCADE,
    message_id      UUID REFERENCES messages(id) ON DELETE SET NULL,
    payload         JSONB NOT NULL,
    attempts        INTEGER NOT NULL DEFAULT 0,
    max_attempts    INTEGER NOT NULL DEFAULT 3,
    scheduled_for   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    processed_at    TIMESTAMPTZ,
    failed_at       TIMESTAMPTZ,
    error           TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_queue_jobs_campaign ON message_queue_jobs(campaign_id);
CREATE INDEX idx_queue_jobs_scheduled ON message_queue_jobs(scheduled_for) WHERE processed_at IS NULL;
