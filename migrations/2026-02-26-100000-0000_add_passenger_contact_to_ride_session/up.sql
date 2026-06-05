-- Add passenger contact fields to ride_session table
ALTER TABLE ride_session
ADD COLUMN passenger_name TEXT,
ADD COLUMN passenger_phone TEXT;
