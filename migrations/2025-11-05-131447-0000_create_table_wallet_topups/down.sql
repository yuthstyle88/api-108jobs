-- Drop indexes first (if exist)
DROP INDEX IF EXISTS idx_wallet_topups_created_at;
DROP INDEX IF EXISTS idx_wallet_topups_status;
DROP INDEX IF EXISTS idx_wallet_topups_user_id;

-- Drop table
DROP TABLE IF EXISTS wallet_topups;

-- Drop enum type if you created one
DROP TYPE IF EXISTS topup_status;
