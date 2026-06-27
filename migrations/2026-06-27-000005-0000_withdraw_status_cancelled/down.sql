-- Postgres does not support removing enum values; this migration cannot be reversed.
-- Existing Cancelled rows would need to be back-filled to Pending before a clean rollback.
SELECT 1;
