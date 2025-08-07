-- Drop wallet system
DROP TRIGGER IF EXISTS trigger_wallet_version_update ON wallet;
DROP FUNCTION IF EXISTS increment_wallet_version();
DROP INDEX IF EXISTS idx_local_user_wallet_id;
ALTER TABLE local_user DROP COLUMN IF EXISTS wallet_id;
DROP TABLE IF EXISTS wallet;