-- Reverse of 2026-05-25-180000-0000_add_wallet_versioning_and_hold_ledger.
-- DROPs the ledger table and its indexes, removes the version column.
-- Existing wallet data is preserved.

DROP INDEX IF EXISTS uq_wallet_hold_idem;
DROP INDEX IF EXISTS uq_wallet_hold_active_per_billing;
DROP INDEX IF EXISTS idx_wallet_hold_status;
DROP INDEX IF EXISTS idx_wallet_hold_billing;
DROP INDEX IF EXISTS idx_wallet_hold_wallet;

DROP TABLE IF EXISTS wallet_hold;

ALTER TABLE wallet DROP COLUMN IF EXISTS version;
