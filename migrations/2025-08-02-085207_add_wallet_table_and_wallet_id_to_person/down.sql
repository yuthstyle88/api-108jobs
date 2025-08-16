-- Drop index
DROP INDEX IF EXISTS idx_local_user_wallet_id;

-- Remove wallet_id column from local_user table
ALTER TABLE local_user DROP COLUMN IF EXISTS wallet_id;

-- Drop wallet table
DROP TABLE IF EXISTS wallet;