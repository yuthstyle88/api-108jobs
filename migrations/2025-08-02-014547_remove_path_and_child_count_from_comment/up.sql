-- Remove path and child_count columns from comment table
ALTER TABLE comment DROP COLUMN IF EXISTS path;
ALTER TABLE comment DROP COLUMN IF EXISTS child_count;