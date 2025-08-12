-- Create table for billing milestones (installment payments)
CREATE TABLE IF NOT EXISTS billing_milestones (
  id            SERIAL PRIMARY KEY,
  billing_id    INT NOT NULL REFERENCES billing(id) ON DELETE CASCADE,
  seq           INT NOT NULL,
  name          TEXT NOT NULL,
  amount        NUMERIC(12,2) NOT NULL CHECK (amount > 0),
  status        TEXT NOT NULL DEFAULT 'Draft', -- Draft|Submitted|Approved|Released
  submitted_at  TIMESTAMPTZ NOT NULL,
  approved_at   TIMESTAMPTZ,
  released_at   TIMESTAMPTZ
);

-- Ensure sequence number is unique per billing
CREATE UNIQUE INDEX IF NOT EXISTS billing_milestones_billing_seq_uidx ON billing_milestones (billing_id, seq);
