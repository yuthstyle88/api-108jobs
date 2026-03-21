-- Create table for job budget plan with installments
-- installments example:
-- [
--   { "idx": 1, "amount": 500000, "status": "paid" },
--   { "idx": 2, "amount": 300000, "status": "unpaid" },
--   { "idx": 3, "amount": 200000, "status": "unpaid" }
-- ]
CREATE TABLE IF NOT EXISTS job_budget_plan (
  id            SERIAL PRIMARY KEY,
  post_id       INTEGER NOT NULL REFERENCES post(id) ON DELETE CASCADE,
  total_amount  INTEGER NOT NULL CHECK (total_amount >= 0),
  installments  JSONB NOT NULL DEFAULT '[]' CHECK (jsonb_typeof(installments) = 'array'),
  created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
  updated_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- One plan per post
CREATE UNIQUE INDEX IF NOT EXISTS idx_job_budget_plan_post_unique ON job_budget_plan(post_id);
