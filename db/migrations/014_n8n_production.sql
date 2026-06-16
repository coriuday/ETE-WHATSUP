-- ============================================================
-- Migration 014: n8n Production Readiness
-- ============================================================
-- Adds webhook delivery logging for n8n automation events.
-- n8n itself should be configured to use PostgreSQL via env
-- vars in docker-compose (DB_TYPE=postgresdb).
-- ============================================================

CREATE TABLE IF NOT EXISTS webhook_delivery_logs (
    id              UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    org_id          UUID REFERENCES organizations(id) ON DELETE SET NULL,
    trigger_id      UUID REFERENCES automation_triggers(id) ON DELETE SET NULL,
    event_type      VARCHAR(100) NOT NULL,
    payload         JSONB NOT NULL DEFAULT '{}',
    response_status INTEGER,
    response_body   TEXT,
    delivered_at    TIMESTAMPTZ,
    error           TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_webhook_logs_org_id    ON webhook_delivery_logs(org_id);
CREATE INDEX idx_webhook_logs_trigger   ON webhook_delivery_logs(trigger_id);
CREATE INDEX idx_webhook_logs_created   ON webhook_delivery_logs(created_at DESC);
CREATE INDEX idx_webhook_logs_event     ON webhook_delivery_logs(event_type);
