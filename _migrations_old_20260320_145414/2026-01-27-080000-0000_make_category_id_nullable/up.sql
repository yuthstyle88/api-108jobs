-- Make category_id nullable for posts
-- This allows delivery posts to exist without a category, relying on post_kind for distinction

-- First, check if the column is already nullable by trying to alter it
-- If it fails, we ignore the error (already nullable)
DO $$
BEGIN
  ALTER TABLE post ALTER COLUMN category_id DROP NOT NULL;
EXCEPTION
  WHEN others THEN
    -- Column might already be nullable, ignore error
    NULL;
END $$;
