ALTER TABLE workflow
    ADD COLUMN IF NOT EXISTS status_before_cancel workflow_status;