-- Add proposal fields to comment table to support proposal functionality
ALTER TABLE comment 
ADD COLUMN budget int,
ADD COLUMN working_days int,
ADD COLUMN brief_url text;
