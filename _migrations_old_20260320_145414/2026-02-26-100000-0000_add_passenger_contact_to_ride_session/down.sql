-- Remove passenger contact fields from ride_session table
ALTER TABLE ride_session
DROP COLUMN passenger_phone,
DROP COLUMN passenger_name;
