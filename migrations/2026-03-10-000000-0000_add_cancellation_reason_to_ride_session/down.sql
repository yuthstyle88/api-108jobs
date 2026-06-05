-- Remove cancellation_reason column from ride_session table
ALTER TABLE ride_session
DROP COLUMN IF EXISTS cancellation_reason;
