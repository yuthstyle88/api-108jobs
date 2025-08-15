-- Update language_profile table to use level_id instead of level_name
-- Map the enum values to numeric levels: 1=Low, 2=Medium, 3=High

-- Add the new level_id column
ALTER TABLE language_profile ADD COLUMN level_id INTEGER;

-- Update existing data with mappings
UPDATE language_profile SET level_id = CASE
  WHEN level_name IN ('beginner', 'pre_intermediate') THEN 1  -- Low
  WHEN level_name IN ('intermediate', 'upper_intermediate') THEN 2  -- Medium  
  WHEN level_name IN ('advanced', 'near_native', 'native') THEN 3  -- High
  ELSE 1  -- Default to Low for any unexpected values
END;

-- Make level_id NOT NULL now that it has values
ALTER TABLE language_profile ALTER COLUMN level_id SET NOT NULL;

-- Drop the old level_name column
ALTER TABLE language_profile DROP COLUMN level_name;

-- Add constraint to ensure valid level values
ALTER TABLE language_profile ADD CONSTRAINT language_profile_level_id_check CHECK (level_id >= 1 AND level_id <= 3);