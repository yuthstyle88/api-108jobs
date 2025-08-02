-- Remove ap_id constraint and column from comment table
ALTER TABLE comment DROP CONSTRAINT IF EXISTS idx_comment_ap_id;
ALTER TABLE comment DROP COLUMN IF EXISTS ap_id;
