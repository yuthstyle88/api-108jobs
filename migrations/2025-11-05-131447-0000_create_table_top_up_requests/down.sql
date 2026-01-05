-- Drop indexes first (if exist)
DROP INDEX IF EXISTS idx_top_up_requests_user_id;
DROP INDEX IF EXISTS idx_top_up_requests_status;
DROP INDEX IF EXISTS idx_top_up_requests_user_id;

-- Drop table
DROP TABLE IF EXISTS top_up_requests;

-- Drop enum type if you created one
DROP TYPE IF EXISTS top_up_status;
