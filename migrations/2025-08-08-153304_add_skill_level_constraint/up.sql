-- Add constraint to ensure skill level_id is between 1 and 5
ALTER TABLE skills 
ADD CONSTRAINT skills_level_id_check 
CHECK (level_id IS NULL OR (level_id >= 1 AND level_id <= 5));
