-- Drop indices and table for wallet transactions
DROP INDEX IF EXISTS idx_wallet_tx_ref_time;
DROP INDEX IF EXISTS idx_wallet_tx_wallet_time;
DROP INDEX IF EXISTS idx_wallet_tx_idem;
DROP TABLE IF EXISTS wallet_transaction;

-- Drop platform index and columns
DROP INDEX IF EXISTS idx_wallet_platform;
ALTER TABLE wallet
  DROP COLUMN IF EXISTS balance_outstanding,
  DROP COLUMN IF EXISTS balance_available,
  DROP COLUMN IF EXISTS balance_total,
  DROP COLUMN IF EXISTS is_platform;