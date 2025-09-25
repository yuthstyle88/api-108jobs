ALTER TABLE workflow
    ADD COLUMN billing_id INTEGER NULL REFERENCES billing(id);

CREATE INDEX IF NOT EXISTS idx_workflow_billing_id ON workflow(billing_id);