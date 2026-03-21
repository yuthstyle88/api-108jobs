DROP INDEX IF EXISTS idx_person_portfolio_pics;
DROP INDEX IF EXISTS idx_person_work_samples;

ALTER TABLE person
DROP COLUMN IF EXISTS work_samples,
DROP COLUMN IF EXISTS portfolio_pics;