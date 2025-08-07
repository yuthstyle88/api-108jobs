-- Remove country field from person table
DROP INDEX IF EXISTS idx_person_country;
ALTER TABLE person DROP COLUMN IF EXISTS country;