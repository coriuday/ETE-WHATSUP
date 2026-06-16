-- ============================================================
-- Migration 013: Message Queue Jobs — Status Column
-- ============================================================
-- Adds explicit status to message_queue_jobs for the worker
-- pool to manage job lifecycle.
-- ============================================================

CREATE TYPE queue_job_status AS ENUM (
    'pending',
    'processing',
    'sent',
    'failed',
    'retry'
);

ALTER TABLE message_queue_jobs
    ADD COLUMN IF NOT EXISTS status queue_job_status NOT NULL DEFAULT 'pending',
    ADD COLUMN IF NOT EXISTS retry_at TIMESTAMPTZ;  -- when to next attempt (NULL = not retrying)

-- Index for worker to pull pending/retry jobs efficiently
CREATE INDEX IF NOT EXISTS idx_queue_jobs_status_scheduled
    ON message_queue_jobs(status, scheduled_for)
    WHERE status IN ('pending', 'retry');

-- Index to check campaign completion (all jobs done)
CREATE INDEX IF NOT EXISTS idx_queue_jobs_campaign_status
    ON message_queue_jobs(campaign_id, status);
