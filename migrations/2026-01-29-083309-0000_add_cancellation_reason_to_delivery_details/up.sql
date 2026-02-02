-- Add cancellation_reason column to delivery_details table
ALTER TABLE delivery_details
ADD COLUMN cancellation_reason TEXT;
