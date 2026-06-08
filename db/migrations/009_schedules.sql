-- ============================================================
-- Migration 009: Scheduled Campaigns & Recurring Schedules
-- ============================================================

CREATE TYPE schedule_frequency AS ENUM (
    'once',
    'daily',
    'weekly',
    'monthly',
    'custom'        -- arbitrary cron expression
);

CREATE TYPE schedule_status AS ENUM (
    'active',
    'paused',
    'completed',
    'cancelled'
);

-- ── Campaign Schedules ───────────────────────────────────────
CREATE TABLE IF NOT EXISTS campaign_schedules (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id     UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    campaign_id         UUID NOT NULL REFERENCES campaigns(id) ON DELETE CASCADE,

    -- Timing
    frequency           schedule_frequency NOT NULL DEFAULT 'once',
    cron_expression     VARCHAR(100),           -- for custom frequency
    timezone            VARCHAR(100) NOT NULL DEFAULT 'Asia/Kolkata',
    next_run_at         TIMESTAMPTZ NOT NULL,
    last_run_at         TIMESTAMPTZ,

    -- For recurring
    run_count           INTEGER NOT NULL DEFAULT 0,
    max_runs            INTEGER,                -- NULL = unlimited

    -- Status
    status              schedule_status NOT NULL DEFAULT 'active',

    -- End condition
    ends_at             TIMESTAMPTZ,            -- stop recurring after this date

    created_by          UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_schedules_org_id ON campaign_schedules(organization_id);
CREATE INDEX idx_schedules_campaign_id ON campaign_schedules(campaign_id);
CREATE INDEX idx_schedules_next_run ON campaign_schedules(next_run_at) WHERE status = 'active';

CREATE TRIGGER update_campaign_schedules_updated_at
    BEFORE UPDATE ON campaign_schedules
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ── Schedule Run History ─────────────────────────────────────
CREATE TABLE IF NOT EXISTS schedule_run_history (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    schedule_id     UUID NOT NULL REFERENCES campaign_schedules(id) ON DELETE CASCADE,
    campaign_id     UUID NOT NULL REFERENCES campaigns(id) ON DELETE CASCADE,
    run_number      INTEGER NOT NULL,
    started_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at    TIMESTAMPTZ,
    status          VARCHAR(50) NOT NULL DEFAULT 'running',   -- running, completed, failed
    msgs_sent       INTEGER NOT NULL DEFAULT 0,
    error           TEXT
);

CREATE INDEX idx_schedule_history_schedule_id ON schedule_run_history(schedule_id);

-- ── n8n Automation Triggers ──────────────────────────────────
CREATE TABLE IF NOT EXISTS automation_triggers (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name            VARCHAR(255) NOT NULL,
    event_type      VARCHAR(100) NOT NULL,      -- e.g. 'contact.created', 'campaign.completed'
    n8n_webhook_url TEXT NOT NULL,
    is_active       BOOLEAN NOT NULL DEFAULT TRUE,
    last_triggered_at TIMESTAMPTZ,
    trigger_count   INTEGER NOT NULL DEFAULT 0,
    created_by      UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_automation_triggers_org_id ON automation_triggers(organization_id);
CREATE INDEX idx_automation_triggers_event ON automation_triggers(event_type);
