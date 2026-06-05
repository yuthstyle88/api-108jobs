-- Add cancellation_reason column to ride_session table
ALTER TABLE ride_session
ADD COLUMN cancellation_reason TEXT;
