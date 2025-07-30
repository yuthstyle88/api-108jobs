DROP INDEX IF EXISTS idx_job_posts_creator_id;
DROP INDEX IF EXISTS idx_job_posts_service_catalog_id;
DROP INDEX IF EXISTS idx_job_posts_working_from;

DROP TRIGGER IF EXISTS trig_update_job_posts_updated_at ON job_posts;

DROP FUNCTION IF EXISTS update_updated_at();

DROP TABLE IF EXISTS job_posts;