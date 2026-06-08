-- ============================================================
-- Migration 004: Contacts & Groups
-- ============================================================

CREATE TYPE contact_status AS ENUM ('active', 'blocked', 'unsubscribed', 'invalid');
CREATE TYPE contact_source AS ENUM ('manual', 'csv_import', 'excel_import', 'api', 'form', 'whatsapp_inbound');

-- ── Contact Groups ───────────────────────────────────────────
CREATE TABLE IF NOT EXISTS contact_groups (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name            VARCHAR(255) NOT NULL,
    description     TEXT,
    color           VARCHAR(7) DEFAULT '#6366f1',   -- hex color for UI
    contact_count   INTEGER NOT NULL DEFAULT 0,      -- denormalized for performance
    created_by      UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_contact_groups_org_id ON contact_groups(organization_id);

CREATE TRIGGER update_contact_groups_updated_at
    BEFORE UPDATE ON contact_groups
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ── Contacts ─────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS contacts (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,

    -- Identity
    phone_number    VARCHAR(20) NOT NULL,            -- E.164 format
    first_name      VARCHAR(100),
    last_name       VARCHAR(100),
    email           VARCHAR(255),
    avatar_url      TEXT,

    -- WhatsApp status
    wa_status       contact_status NOT NULL DEFAULT 'active',
    wa_opted_in     BOOLEAN NOT NULL DEFAULT TRUE,
    wa_opted_in_at  TIMESTAMPTZ,
    opted_out_at    TIMESTAMPTZ,

    -- Segmentation
    tags            TEXT[] NOT NULL DEFAULT '{}',
    source          contact_source NOT NULL DEFAULT 'manual',

    -- Custom fields (flexible key-value)
    custom_fields   JSONB NOT NULL DEFAULT '{}',

    -- Stats
    total_msgs_received     INTEGER NOT NULL DEFAULT 0,
    total_msgs_sent         INTEGER NOT NULL DEFAULT 0,
    last_contacted_at       TIMESTAMPTZ,
    last_replied_at         TIMESTAMPTZ,

    -- Import tracking
    imported_from   TEXT,                           -- filename if CSV/Excel
    imported_at     TIMESTAMPTZ,
    created_by      UUID REFERENCES users(id) ON DELETE SET NULL,

    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at      TIMESTAMPTZ,

    UNIQUE (organization_id, phone_number)
);

CREATE INDEX idx_contacts_org_id ON contacts(organization_id);
CREATE INDEX idx_contacts_phone ON contacts(phone_number);
CREATE INDEX idx_contacts_status ON contacts(wa_status);
CREATE INDEX idx_contacts_tags ON contacts USING gin(tags);
CREATE INDEX idx_contacts_custom_fields ON contacts USING gin(custom_fields);
CREATE INDEX idx_contacts_deleted_at ON contacts(deleted_at) WHERE deleted_at IS NULL;

CREATE TRIGGER update_contacts_updated_at
    BEFORE UPDATE ON contacts
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ── Contact Group Memberships ────────────────────────────────
CREATE TABLE IF NOT EXISTS contact_group_members (
    contact_id      UUID NOT NULL REFERENCES contacts(id) ON DELETE CASCADE,
    group_id        UUID NOT NULL REFERENCES contact_groups(id) ON DELETE CASCADE,
    added_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (contact_id, group_id)
);

CREATE INDEX idx_cgm_group_id ON contact_group_members(group_id);
CREATE INDEX idx_cgm_contact_id ON contact_group_members(contact_id);

-- ── Segments (dynamic, query-based) ─────────────────────────
CREATE TABLE IF NOT EXISTS contact_segments (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name            VARCHAR(255) NOT NULL,
    description     TEXT,
    filter_rules    JSONB NOT NULL DEFAULT '{}',    -- filter criteria as JSON
    contact_count   INTEGER NOT NULL DEFAULT 0,
    last_computed_at TIMESTAMPTZ,
    created_by      UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_segments_org_id ON contact_segments(organization_id);

-- ── Import Jobs ──────────────────────────────────────────────
CREATE TYPE import_status AS ENUM ('pending', 'processing', 'completed', 'failed');

CREATE TABLE IF NOT EXISTS contact_imports (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    file_name       VARCHAR(500) NOT NULL,
    file_url        TEXT NOT NULL,
    status          import_status NOT NULL DEFAULT 'pending',
    total_rows      INTEGER,
    processed_rows  INTEGER NOT NULL DEFAULT 0,
    imported_count  INTEGER NOT NULL DEFAULT 0,
    skipped_count   INTEGER NOT NULL DEFAULT 0,
    error_count     INTEGER NOT NULL DEFAULT 0,
    errors          JSONB,                          -- array of error details
    created_by      UUID REFERENCES users(id) ON DELETE SET NULL,
    started_at      TIMESTAMPTZ,
    completed_at    TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_contact_imports_org_id ON contact_imports(organization_id);
CREATE INDEX idx_contact_imports_status ON contact_imports(status);
