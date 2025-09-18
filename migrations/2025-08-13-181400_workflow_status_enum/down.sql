-- Revert workflow objects in correct dependency order
DROP INDEX IF EXISTS one_active_workflow_per_post;
-- 2) Drop table referencing the enum
DROP TABLE IF EXISTS workflow;
-- 3) Finally drop the enum type
DROP TYPE IF EXISTS workflow_status;