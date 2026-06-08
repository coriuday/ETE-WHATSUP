-- ============================================================
-- Migration 005: Campaigns
-- ============================================================

CREATE TYPE campaign_type AS ENUM (
    'bulk_message',         -- one-time blast to a group/segment
    'drip',                 -- multi-step sequence over time
    'transactional',        -- API-triggered per-user messages
    'ab_test'               -- A/B test between variants
);

CREATE TYPE campaign_status AS ENUM (
    'draft',
    'scheduled',
    'running',
    'paused',
    'completed',
    'failed',
    'cancelled'
);

CREATE TYPE campaign_target_type AS ENUM (
    'group',
    'segment',
    'all_contacts',
    'custom_list'           -- manually selected contact IDs
);

-- ── Campaigns ────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS campaigns (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id     UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    wa_account_id       UUID REFERENCES whatsapp_accounts(id) ON DELETE SET NULL,

    -- Identity
    name                VARCHAR(500) NOT NULL,
    description         TEXT,
    type                campaign_type NOT NULL DEFAULT 'bulk_message',
    status              campaign_status NOT NULL DEFAULT 'draft',

    -- Target audience
    target_type         campaign_target_type NOT NULL DEFAULT 'group',
    target_group_id     UUID REFERENCES contact_groups(id) ON DELETE SET NULL,
    target_segment_id   UUID REFERENCES contact_segments(id) ON DELETE SET NULL,
    target_contact_ids  UUID[],                         -- for custom_list type

    -- Content (template-based or freeform)
    template_id         UUID,                           -- FK added in 007
    message_body        TEXT,                           -- freeform text (non-template)
    media_url           TEXT,                           -- image/video/doc URL
    media_type          VARCHAR(20),                    -- image, video, document, audio
    buttons             JSONB,                          -- quick reply / CTA buttons

    -- A/B Test variants
    variants            JSONB,                          -- array of variant definitions

    -- Scheduling
    scheduled_at        TIMESTAMPTZ,
    send_rate           INTEGER NOT NULL DEFAULT 60,    -- messages per minute
    timezone            VARCHAR(100) NOT NULL DEFAULT 'Asia/Kolkata',

    -- Execution tracking
    total_recipients    INTEGER NOT NULL DEFAULT 0,
    sent_count          INTEGER NOT NULL DEFAULT 0,
    delivered_count     INTEGER NOT NULL DEFAULT 0,
    read_count          INTEGER NOT NULL DEFAULT 0,
    failed_count        INTEGER NOT NULL DEFAULT 0,
    reply_count         INTEGER NOT NULL DEFAULT 0,

    -- Timestamps
    started_at          TIMESTAMPTZ,
    completed_at        TIMESTAMPTZ,
    created_by          UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at          TIMESTAMPTZ
);

CREATE INDEX idx_campaigns_org_id ON campaigns(organization_id);
CREATE INDEX idx_campaigns_status ON campaigns(status);
CREATE INDEX idx_campaigns_type ON campaigns(type);
CREATE INDEX idx_campaigns_scheduled_at ON campaigns(scheduled_at) WHERE scheduled_at IS NOT NULL;
CREATE INDEX idx_campaigns_deleted_at ON campaigns(deleted_at) WHERE deleted_at IS NULL;

CREATE TRIGGER update_campaigns_updated_at
    BEFORE UPDATE ON campaigns
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ── Campaign Activity Log ────────────────────────────────────
CREATE TABLE IF NOT EXISTS campaign_activity_logs (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    campaign_id     UUID NOT NULL REFERENCES campaigns(id) ON DELETE CASCADE,
    user_id         UUID REFERENCES users(id) ON DELETE SET NULL,
    action          VARCHAR(100) NOT NULL,   -- e.g. 'created', 'launched', 'paused'
    details         JSONB,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_campaign_logs_campaign_id ON campaign_activity_logs(campaign_id);
