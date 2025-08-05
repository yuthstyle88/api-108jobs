-- Remove country field from local_user table
DROP INDEX IF EXISTS idx_local_user_country;
ALTER TABLE local_user DROP CONSTRAINT IF EXISTS check_country_valid;
ALTER TABLE local_user DROP COLUMN IF EXISTS country;