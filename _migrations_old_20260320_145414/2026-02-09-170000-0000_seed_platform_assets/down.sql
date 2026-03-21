-- Rollback platform assets seeding

-- Note: We do NOT delete the platform wallet or coin in rollback
-- as this could cause data integrity issues. Instead, we only
-- remove the singleton constraint that was added.

-- The platform assets will persist even after rollback to prevent
-- accidental deletion of critical system data.

-- Remove the singleton constraint
DROP INDEX IF EXISTS idx_wallet_platform_singleton;

-- Restore the old partial index (for compatibility)
CREATE INDEX idx_wallet_platform ON wallet (is_platform) WHERE is_platform = TRUE;
