-- Add back path and child_count columns to comment table
ALTER TABLE person
    DROP COLUMN public_key,
    DROP COLUMN private_key;
