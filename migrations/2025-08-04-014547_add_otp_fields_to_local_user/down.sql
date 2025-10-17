-- Add back path and child_count columns to comment table
ALTER TABLE person
   DROP COLUMN IF EXISTS shared_key,
   DROP COLUMN IF EXISTS private_key;
