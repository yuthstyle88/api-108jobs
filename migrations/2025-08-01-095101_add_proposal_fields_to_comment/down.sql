-- Remove proposal fields from comment table
ALTER TABLE comment 
DROP COLUMN budget,
DROP COLUMN working_days,
DROP COLUMN brief_url;
