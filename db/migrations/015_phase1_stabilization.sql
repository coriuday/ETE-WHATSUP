-- ============================================================
-- Migration 015: Phase 1 Architecture Stabilization
-- ============================================================

-- Queue job updated_at for stuck-job reclaim
ALTER TABLE message_queue_jobs
    ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();

CREATE OR REPLACE FUNCTION update_message_queue_jobs_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_message_queue_jobs_updated_at ON message_queue_jobs;
CREATE TRIGGER trg_message_queue_jobs_updated_at
    BEFORE UPDATE ON message_queue_jobs
    FOR EACH ROW EXECUTE FUNCTION update_message_queue_jobs_updated_at();

CREATE INDEX IF NOT EXISTS idx_queue_jobs_processing_updated
    ON message_queue_jobs(status, updated_at)
    WHERE status = 'processing';

-- Webhook idempotency: external Meta event id
ALTER TABLE wa_webhook_events
    ADD COLUMN IF NOT EXISTS external_id VARCHAR(255),
    ADD COLUMN IF NOT EXISTS organization_id UUID REFERENCES organizations(id) ON DELETE SET NULL;

CREATE UNIQUE INDEX IF NOT EXISTS idx_wa_webhook_events_external_id
    ON wa_webhook_events(external_id)
    WHERE external_id IS NOT NULL;

-- Message idempotency on Meta wamid
CREATE UNIQUE INDEX IF NOT EXISTS idx_messages_wa_message_id_unique
    ON messages(wa_message_id)
    WHERE wa_message_id IS NOT NULL;
