-- Rename delivery_status enum to trip_status safely

-- Step 0: Drop problematic partial index (MUST DO FIRST)
DROP INDEX IF EXISTS idx_delivery_details_assigned_rider;

-- Step 1: Create new enum type
CREATE TYPE trip_status AS ENUM (
  'Pending',
  'Assigned',
  'RiderConfirmed',
  'EnRouteToPickup',
  'PickedUp',
  'EnRouteToDropoff',
  'Delivered',
  'Cancelled'
);

-- Step 2: Drop default constraints
ALTER TABLE delivery_details ALTER COLUMN status DROP DEFAULT;
ALTER TABLE ride_session ALTER COLUMN status DROP DEFAULT;

-- Step 3: Convert enum -> TEXT (break dependency)
ALTER TABLE delivery_details
ALTER COLUMN status TYPE TEXT;

ALTER TABLE ride_session
ALTER COLUMN status TYPE TEXT;

-- Step 4: Convert TEXT -> new enum
ALTER TABLE delivery_details
ALTER COLUMN status TYPE trip_status
  USING status::trip_status;

ALTER TABLE ride_session
ALTER COLUMN status TYPE trip_status
  USING status::trip_status;

-- Step 5: Restore defaults
ALTER TABLE delivery_details
    ALTER COLUMN status SET DEFAULT 'Pending'::trip_status;

ALTER TABLE ride_session
    ALTER COLUMN status SET DEFAULT 'Pending'::trip_status;

-- Step 6: Recreate partial index with new enum
CREATE INDEX idx_delivery_details_assigned_rider
    ON delivery_details (assigned_rider_id)
    WHERE status = 'Assigned'::trip_status;

-- Step 7: Drop old enum
DROP TYPE delivery_status;