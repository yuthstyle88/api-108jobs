-- Revert: add back instance_id column
ALTER TABLE post
    ADD COLUMN IF NOT EXISTS instance_id integer;

-- Populate instance_id from category (if category exists)
UPDATE post p
SET instance_id = c.instance_id
FROM category c
WHERE p.category_id = c.id AND p.instance_id IS NULL;

-- Add foreign key constraint (if not exists)
DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM information_schema.table_constraints
    WHERE constraint_name = 'post_instance_id_fkey'
    AND table_name = 'post'
  ) THEN
    ALTER TABLE ONLY post
        ADD CONSTRAINT post_instance_id_fkey FOREIGN KEY (instance_id) REFERENCES instance (id) ON UPDATE CASCADE ON DELETE CASCADE;
  END IF;
END $$;
