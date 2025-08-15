-- Rollback: change language_profile back to use level_name enum

-- First recreate the enum type if it doesn't exist
DO $$ BEGIN
  CREATE TYPE language_level AS ENUM (
    'native',
    'near_native', 
    'advanced',
    'upper_intermediate',
    'intermediate',
    'pre_intermediate',
    'beginner'
  );
EXCEPTION
  WHEN duplicate_object THEN NULL;
END $$;

-- Add back the level_name column
ALTER TABLE language_profile ADD COLUMN level_name language_level;

-- Convert level_id back to level_name
UPDATE language_profile SET level_name = CASE
  WHEN level_id = 1 THEN 'beginner'::language_level
  WHEN level_id = 2 THEN 'intermediate'::language_level  
  WHEN level_id = 3 THEN 'advanced'::language_level
  ELSE 'beginner'::language_level
END;

-- Make level_name NOT NULL and set default
ALTER TABLE language_profile ALTER COLUMN level_name SET NOT NULL;
ALTER TABLE language_profile ALTER COLUMN level_name SET DEFAULT 'beginner'::language_level;

-- Drop the constraint and level_id column
ALTER TABLE language_profile DROP CONSTRAINT IF EXISTS language_profile_level_id_check;
ALTER TABLE language_profile DROP COLUMN level_id;