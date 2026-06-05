-- Remove cancellation_reason column from delivery_details table
ALTER TABLE delivery_details
DROP COLUMN IF EXISTS cancellation_reason;
