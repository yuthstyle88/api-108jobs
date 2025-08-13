-- Reverse migration: Make expires_date column NOT NULL again
-- WARNING: This will fail if any certificates have NULL expires_date values

-- First set any NULL expires_date to a far future date before making it NOT NULL
UPDATE certificates 
SET expires_date = '2099-12-31'::date 
WHERE expires_date IS NULL;

-- Now make the column NOT NULL
ALTER TABLE certificates 
ALTER COLUMN expires_date SET NOT NULL;