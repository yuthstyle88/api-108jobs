-- Revert identity card fields to NOT NULL
ALTER TABLE identity_card 
ALTER COLUMN issued_date SET NOT NULL,
ALTER COLUMN expiry_date SET NOT NULL,
ALTER COLUMN full_name SET NOT NULL,
ALTER COLUMN date_of_birth SET NOT NULL,
ALTER COLUMN nationality SET NOT NULL;