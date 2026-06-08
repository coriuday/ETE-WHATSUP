-- ============================================================
-- Migration 002: Organizations & Members
-- ============================================================

CREATE TYPE org_plan AS ENUM ('free', 'starter', 'professional', 'enterprise');
CREATE TYPE org_status AS ENUM ('active', 'suspended', 'cancelled');
CREATE TYPE member_role AS ENUM ('owner', 'admin', 'member', 'viewer');
CREATE TYPE invitation_status AS ENUM ('pending', 'accepted', 'expired', 'cancelled');

-- ── Organizations ────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS organizations (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name            VARCHAR(255) NOT NULL,
    slug            VARCHAR(100) NOT NULL UNIQUE,   -- URL-safe identifier
    logo_url        TEXT,
    website         TEXT,
    industry        VARCHAR(100),
    country         VARCHAR(100),
    timezone        VARCHAR(100) NOT NULL DEFAULT 'Asia/Kolkata',
    plan            org_plan NOT NULL DEFAULT 'free',
    status          org_status NOT NULL DEFAULT 'active',

    -- Limits (overridable per plan)
    max_contacts        INTEGER NOT NULL DEFAULT 1000,
    max_campaigns       INTEGER NOT NULL DEFAULT 10,
    max_team_members    INTEGER NOT NULL DEFAULT 3,
    max_wa_accounts     INTEGER NOT NULL DEFAULT 1,
    monthly_msg_quota   INTEGER NOT NULL DEFAULT 1000,

    -- Usage tracking (reset monthly)
    msgs_sent_this_month    INTEGER NOT NULL DEFAULT 0,
    quota_reset_at          TIMESTAMPTZ NOT NULL DEFAULT date_trunc('month', NOW()) + INTERVAL '1 month',

    -- Owner
    owner_id        UUID NOT NULL REFERENCES users(id),

    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at      TIMESTAMPTZ
);

CREATE INDEX idx_organizations_slug ON organizations(slug);
CREATE INDEX idx_organizations_owner_id ON organizations(owner_id);
CREATE INDEX idx_organizations_status ON organizations(status);

CREATE TRIGGER update_organizations_updated_at
    BEFORE UPDATE ON organizations
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ── Organization Members (many-to-many with roles) ───────────
CREATE TABLE IF NOT EXISTS org_members (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role            member_role NOT NULL DEFAULT 'member',
    joined_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (organization_id, user_id)
);

CREATE INDEX idx_org_members_org_id ON org_members(organization_id);
CREATE INDEX idx_org_members_user_id ON org_members(user_id);

-- ── Invitations ──────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS org_invitations (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    invited_by      UUID NOT NULL REFERENCES users(id),
    email           VARCHAR(255) NOT NULL,
    role            member_role NOT NULL DEFAULT 'member',
    token           VARCHAR(255) NOT NULL UNIQUE,
    status          invitation_status NOT NULL DEFAULT 'pending',
    expires_at      TIMESTAMPTZ NOT NULL DEFAULT NOW() + INTERVAL '7 days',
    accepted_at     TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_org_invitations_token ON org_invitations(token);
CREATE INDEX idx_org_invitations_email ON org_invitations(email);
CREATE INDEX idx_org_invitations_org_id ON org_invitations(organization_id);
