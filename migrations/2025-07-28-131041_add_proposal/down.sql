DROP INDEX IF EXISTS idx_proposals_service_id;
DROP INDEX IF EXISTS idx_proposals_job_id;
DROP INDEX IF EXISTS idx_proposals_user_job_unique;

DROP TRIGGER IF EXISTS trig_update_proposals_updated_at ON proposals;

DROP FUNCTION IF EXISTS update_updated_at();

DROP TABLE IF EXISTS proposals;