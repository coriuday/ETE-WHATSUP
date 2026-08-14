-- ============================================================
-- Migration 017: Native automation workflows
-- ============================================================

CREATE TABLE IF NOT EXISTS automation_workflows (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name            VARCHAR(255) NOT NULL,
    description     TEXT,
    enabled         BOOLEAN NOT NULL DEFAULT TRUE,
    trigger_type    VARCHAR(64) NOT NULL,
    definition      JSONB NOT NULL DEFAULT '{"steps":[]}'::jsonb,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at      TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_automation_workflows_org ON automation_workflows(organization_id);
CREATE INDEX IF NOT EXISTS idx_automation_workflows_trigger ON automation_workflows(organization_id, trigger_type);

CREATE TABLE IF NOT EXISTS automation_runs (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    workflow_id     UUID NOT NULL REFERENCES automation_workflows(id) ON DELETE CASCADE,
    status          VARCHAR(32) NOT NULL DEFAULT 'running',
    trigger_type    VARCHAR(64) NOT NULL,
    payload         JSONB NOT NULL DEFAULT '{}'::jsonb,
    error           TEXT,
    started_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    finished_at     TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_automation_runs_org ON automation_runs(organization_id, started_at DESC);
CREATE INDEX IF NOT EXISTS idx_automation_runs_workflow ON automation_runs(workflow_id);

CREATE TABLE IF NOT EXISTS assignment_rules (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    name            VARCHAR(255) NOT NULL,
    strategy        VARCHAR(32) NOT NULL DEFAULT 'default',
    default_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    enabled         BOOLEAN NOT NULL DEFAULT TRUE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
