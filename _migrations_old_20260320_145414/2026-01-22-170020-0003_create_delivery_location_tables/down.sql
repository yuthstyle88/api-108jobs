-- down.sql — verbose & defensive variant

-- Drop index
DROP INDEX IF EXISTS idx_delivery_location_history_post_time;

-- Drop history table
DROP TABLE IF EXISTS delivery_location_history;

-- Drop current location table
DROP TABLE IF EXISTS delivery_location_current;

-- Optional: informational notice (visible in migration logs)
DO
$$
BEGIN
  RAISE
NOTICE 'Dropped delivery_location_current and delivery_location_history tables';
  RAISE
NOTICE 'Foreign keys to post(id) and rider(id) were automatically removed';
END $$;