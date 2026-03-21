-- Drop indexes
DROP INDEX IF EXISTS idx_billing_created_at;
DROP INDEX IF EXISTS idx_billing_post_id;
DROP INDEX IF EXISTS idx_billing_employer_id;
DROP INDEX IF EXISTS idx_billing_freelancer_id;

-- Drop billing table
DROP TABLE IF EXISTS billing;

-- Drop billing status enum
DROP TYPE IF EXISTS billing_status;
