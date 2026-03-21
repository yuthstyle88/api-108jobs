-- Drop index first
DROP INDEX IF EXISTS idx_person_wallet_id;

-- Drop column (this also removes FK constraint automatically)
ALTER TABLE person
DROP COLUMN IF EXISTS wallet_id;

-- Drop wallet table
DROP TABLE IF EXISTS wallet;