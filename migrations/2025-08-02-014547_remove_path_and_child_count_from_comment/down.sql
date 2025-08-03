-- Add back path and child_count columns to comment table
ALTER TABLE comment ADD COLUMN path ltree;
ALTER TABLE comment ADD COLUMN child_count int4 NOT NULL DEFAULT 0;