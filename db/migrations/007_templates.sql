-- ============================================================
-- Migration 007: Message Templates
-- ============================================================

CREATE TYPE template_category AS ENUM (
    'marketing',
    'utility',
    'authentication'
);

CREATE TYPE template_status AS ENUM (
    'draft',
    'pending_approval',   -- submitted to Meta
    'approved',
    'rejected',
    'disabled'
);

CREATE TYPE template_language AS ENUM (
    'en',       -- English
    'en_US',    -- English (US)
    'hi',       -- Hindi
    'mr',       -- Marathi
    'gu',       -- Gujarati
    'ta',       -- Tamil
    'te',       -- Telugu
    'kn',       -- Kannada
    'bn',       -- Bengali
    'pa',       -- Punjabi
    'ar',       -- Arabic
    'es',       -- Spanish
    'pt_BR',    -- Portuguese (Brazil)
    'fr',       -- French
    'de',       -- German
    'id',       -- Indonesian
    'ja'        -- Japanese
);

-- ── Templates ────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS templates (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id     UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    wa_account_id       UUID REFERENCES whatsapp_accounts(id) ON DELETE SET NULL,

    -- Meta identifiers
    meta_template_id    VARCHAR(255),           -- Meta's template ID after approval
    name                VARCHAR(512) NOT NULL,  -- Meta template name (must be unique per WABA)
    display_name        VARCHAR(255) NOT NULL,  -- Internal display name
    category            template_category NOT NULL DEFAULT 'marketing',
    language            template_language NOT NULL DEFAULT 'en',
    status              template_status NOT NULL DEFAULT 'draft',

    -- Structure (mirrors Meta template API)
    header              JSONB,   -- { type: 'TEXT'|'IMAGE'|'VIDEO'|'DOCUMENT', text?, example? }
    body_text           TEXT NOT NULL,          -- with {{1}}, {{2}} variable placeholders
    body_example_vars   TEXT[],                 -- example values for placeholders
    footer_text         TEXT,
    buttons             JSONB,                  -- array of button definitions

    -- Variable info
    variable_count      INTEGER NOT NULL DEFAULT 0,
    variable_definitions JSONB NOT NULL DEFAULT '{}',   -- { "1": { label, default } }

    -- Usage
    usage_count         INTEGER NOT NULL DEFAULT 0,

    -- Rejection info
    rejection_reason    TEXT,
    rejected_at         TIMESTAMPTZ,

    -- Audit
    created_by          UUID REFERENCES users(id) ON DELETE SET NULL,
    submitted_at        TIMESTAMPTZ,
    approved_at         TIMESTAMPTZ,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at          TIMESTAMPTZ
);

CREATE INDEX idx_templates_org_id ON templates(organization_id);
CREATE INDEX idx_templates_status ON templates(status);
CREATE INDEX idx_templates_category ON templates(category);
CREATE INDEX idx_templates_deleted_at ON templates(deleted_at) WHERE deleted_at IS NULL;

CREATE TRIGGER update_templates_updated_at
    BEFORE UPDATE ON templates
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ── Add FK from campaigns.template_id to templates ───────────
ALTER TABLE campaigns
    ADD CONSTRAINT fk_campaigns_template
    FOREIGN KEY (template_id) REFERENCES templates(id) ON DELETE SET NULL;
