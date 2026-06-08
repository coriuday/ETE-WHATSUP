-- ============================================================
-- Migration 003: WhatsApp Accounts
-- ============================================================

CREATE TYPE wa_account_status AS ENUM (
    'disconnected',
    'connecting',
    'connected',
    'banned',
    'error'
);

CREATE TYPE wa_account_type AS ENUM (
    'cloud_api',    -- Meta WhatsApp Cloud API (official)
    'business_api'  -- On-premises BSP API
);

-- ── WhatsApp Business Accounts ───────────────────────────────
CREATE TABLE IF NOT EXISTS whatsapp_accounts (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id     UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,

    -- Display
    display_name        VARCHAR(255) NOT NULL,
    phone_number        VARCHAR(20) NOT NULL,       -- E.164 format: +91XXXXXXXXXX
    phone_number_id     VARCHAR(100),               -- Meta phone_number_id
    waba_id             VARCHAR(100),               -- WhatsApp Business Account ID

    -- Connection
    account_type        wa_account_type NOT NULL DEFAULT 'cloud_api',
    status              wa_account_status NOT NULL DEFAULT 'disconnected',

    -- Credentials (AES-256 encrypted at application layer)
    access_token_enc    TEXT,                       -- encrypted Meta API token
    webhook_verify_token VARCHAR(255),

    -- Quality & Limits
    messaging_limit     VARCHAR(50),               -- e.g. "1K", "10K", "100K"
    quality_rating      VARCHAR(20),               -- GREEN, YELLOW, RED
    
    -- Business Profile
    business_name       VARCHAR(255),
    business_description TEXT,
    business_category   VARCHAR(100),
    business_email      VARCHAR(255),
    business_website    TEXT,
    profile_picture_url TEXT,

    -- Stats
    total_msgs_sent     BIGINT NOT NULL DEFAULT 0,
    last_message_at     TIMESTAMPTZ,

    connected_at        TIMESTAMPTZ,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at          TIMESTAMPTZ
);

CREATE INDEX idx_wa_accounts_org_id ON whatsapp_accounts(organization_id);
CREATE INDEX idx_wa_accounts_phone ON whatsapp_accounts(phone_number);
CREATE INDEX idx_wa_accounts_status ON whatsapp_accounts(status);

CREATE TRIGGER update_wa_accounts_updated_at
    BEFORE UPDATE ON whatsapp_accounts
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ── WhatsApp Webhook Events Log ──────────────────────────────
CREATE TABLE IF NOT EXISTS wa_webhook_events (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    account_id      UUID REFERENCES whatsapp_accounts(id) ON DELETE SET NULL,
    event_type      VARCHAR(100) NOT NULL,          -- e.g. 'message', 'status'
    payload         JSONB NOT NULL,
    processed       BOOLEAN NOT NULL DEFAULT FALSE,
    error           TEXT,
    received_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    processed_at    TIMESTAMPTZ
);

CREATE INDEX idx_wa_webhook_events_account ON wa_webhook_events(account_id);
CREATE INDEX idx_wa_webhook_events_processed ON wa_webhook_events(processed);
CREATE INDEX idx_wa_webhook_events_type ON wa_webhook_events(event_type);
