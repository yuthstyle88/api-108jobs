-- This file should undo anything in `up.sql`
ALTER TABLE certificates 
DROP COLUMN achieved_date,
DROP COLUMN expires_date,
DROP COLUMN url;
