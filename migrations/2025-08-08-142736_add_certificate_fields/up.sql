-- Add additional fields to certificates table
ALTER TABLE certificates 
ADD COLUMN achieved_date DATE,
ADD COLUMN expires_date DATE,
ADD COLUMN url TEXT;
