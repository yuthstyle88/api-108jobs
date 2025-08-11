-- Make specific identity card fields nullable for registration
ALTER TABLE identity_card 
ALTER COLUMN issued_date DROP NOT NULL,
ALTER COLUMN expiry_date DROP NOT NULL,
ALTER COLUMN full_name DROP NOT NULL,
ALTER COLUMN date_of_birth DROP NOT NULL,
ALTER COLUMN nationality DROP NOT NULL;