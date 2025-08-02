-- Add back ap_id column to comment table
ALTER TABLE comment ADD COLUMN ap_id TEXT;

-- Create unique constraint on ap_id (restoring previous constraint)
ALTER TABLE comment ADD CONSTRAINT idx_comment_ap_id UNIQUE (ap_id);
