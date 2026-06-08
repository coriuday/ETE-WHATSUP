-- ============================================================
-- Migration 008: Conversations (Inbox / Live Chat)
-- ============================================================

CREATE TYPE conversation_status AS ENUM (
    'open',
    'pending',
    'resolved',
    'snoozed'
);

CREATE TYPE conversation_channel AS ENUM (
    'whatsapp'
    -- future: 'instagram', 'messenger'
);

-- ── Conversations ────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS conversations (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id     UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    wa_account_id       UUID REFERENCES whatsapp_accounts(id) ON DELETE SET NULL,
    contact_id          UUID NOT NULL REFERENCES contacts(id) ON DELETE CASCADE,

    channel             conversation_channel NOT NULL DEFAULT 'whatsapp',
    status              conversation_status NOT NULL DEFAULT 'open',

    -- Customer service window (Meta's 24-hour session)
    session_expires_at  TIMESTAMPTZ,
    is_in_session       BOOLEAN NOT NULL DEFAULT FALSE,

    -- Assignment
    assigned_to         UUID REFERENCES users(id) ON DELETE SET NULL,
    assigned_at         TIMESTAMPTZ,

    -- Metadata
    subject             TEXT,                       -- optional thread subject
    first_message_at    TIMESTAMPTZ,
    last_message_at     TIMESTAMPTZ,
    last_message_body   TEXT,                       -- preview text
    last_message_dir    message_direction,
    unread_count        INTEGER NOT NULL DEFAULT 0,

    -- Labels / tags
    labels              TEXT[] NOT NULL DEFAULT '{}',

    -- Snoozed until
    snoozed_until       TIMESTAMPTZ,

    -- Resolved
    resolved_by         UUID REFERENCES users(id) ON DELETE SET NULL,
    resolved_at         TIMESTAMPTZ,

    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_conversations_org_id ON conversations(organization_id);
CREATE INDEX idx_conversations_contact_id ON conversations(contact_id);
CREATE INDEX idx_conversations_status ON conversations(status);
CREATE INDEX idx_conversations_assigned_to ON conversations(assigned_to);
CREATE INDEX idx_conversations_last_msg ON conversations(last_message_at DESC);
CREATE INDEX idx_conversations_labels ON conversations USING gin(labels);

CREATE TRIGGER update_conversations_updated_at
    BEFORE UPDATE ON conversations
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ── Add FK from messages.conversation_id to conversations ────
ALTER TABLE messages
    ADD CONSTRAINT fk_messages_conversation
    FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE SET NULL;

-- ── Conversation Notes (internal agent notes) ────────────────
CREATE TABLE IF NOT EXISTS conversation_notes (
    id                  UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    conversation_id     UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    created_by          UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    body                TEXT NOT NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_conv_notes_conversation ON conversation_notes(conversation_id);

-- ── Conversation Labels ──────────────────────────────────────
CREATE TABLE IF NOT EXISTS conversation_label_definitions (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name            VARCHAR(100) NOT NULL,
    color           VARCHAR(7) NOT NULL DEFAULT '#6366f1',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (organization_id, name)
);

CREATE INDEX idx_label_defs_org ON conversation_label_definitions(organization_id);
