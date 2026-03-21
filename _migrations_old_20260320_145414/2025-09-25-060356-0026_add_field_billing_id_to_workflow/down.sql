-- This file should undo anything in `up.sql`
DROP INDEX IF EXISTS idx_workflow_billing_id;
ALTER TABLE workflow DROP COLUMN IF EXISTS billing_id;
