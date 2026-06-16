-- ============================================================
-- Migration 012: Contact Import Worker Support
-- ============================================================
-- Adds columns needed by the async import worker to track
-- per-row progress and error details.
-- ============================================================

ALTER TABLE contact_imports
    ADD COLUMN IF NOT EXISTS file_type VARCHAR(10) NOT NULL DEFAULT 'csv',  -- 'csv' | 'xlsx'
    ADD COLUMN IF NOT EXISTS error_details JSONB NOT NULL DEFAULT '[]';      -- array of row-level errors

-- Index to quickly find pending/processing jobs for the worker
CREATE INDEX IF NOT EXISTS idx_contact_imports_status
    ON contact_imports(status, created_at DESC)
    WHERE status IN ('pending', 'processing');
