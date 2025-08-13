-- Make expires_date column nullable in certificates table
-- This allows certificates without expiration dates (e.g., permanent certifications)

ALTER TABLE certificates 
ALTER COLUMN expires_date DROP NOT NULL;