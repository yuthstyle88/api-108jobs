-- Revert workflow objects in correct dependency order
-- 1) Drop unique index
DROP INDEX IF EXISTS idx_workflow_post_unique;
-- 2) Drop table referencing the enum
DROP TABLE IF EXISTS workflow;
-- 3) Finally drop the enum type
DROP TYPE IF EXISTS workflow_status;