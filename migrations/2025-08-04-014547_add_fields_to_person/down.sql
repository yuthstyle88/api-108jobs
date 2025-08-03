-- Add back path and child_count columns to comment table
ALTER TABLE person
    DROP COLUMN public_key text,
    DROP COLUMN private_key text;
