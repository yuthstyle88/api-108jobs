-- Remove redundant wallet_id field from person table
-- Keep wallet.person_id as the single source of truth for person-wallet relationship

DROP INDEX IF EXISTS idx_person_wallet_id;
ALTER TABLE person DROP COLUMN IF EXISTS wallet_id;