-- Add country field to local_user table for region-based bank filtering
ALTER TABLE local_user ADD COLUMN country VARCHAR(100) DEFAULT 'Thailand';

-- Update existing users to Thailand as default (can be changed later)
UPDATE local_user SET country = 'Thailand' WHERE country IS NULL;

-- Make country field not null after updating existing records
ALTER TABLE local_user ALTER COLUMN country SET NOT NULL;

-- Add constraint to only allow Thailand and Vietnam
ALTER TABLE local_user ADD CONSTRAINT check_country_valid 
    CHECK (country IN ('Thailand', 'Vietnam'));

-- Create index for efficient filtering
CREATE INDEX idx_local_user_country ON local_user(country);