-- Revert: make category_id required again
-- Only set NOT NULL if there are no null values
DO $$
BEGIN
  -- First ensure no null values exist
  UPDATE post SET category_id = 1 WHERE category_id IS NULL;

  -- Then set NOT NULL
  ALTER TABLE post ALTER COLUMN category_id SET NOT NULL;
EXCEPTION
  WHEN others THEN
    -- Ignore errors
    NULL;
END $$;
