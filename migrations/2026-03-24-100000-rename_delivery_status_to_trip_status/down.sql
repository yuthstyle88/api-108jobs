-- Revert trip_status back to delivery_status

-- Step 0: Drop index that depends on trip_status
DROP INDEX IF EXISTS idx_delivery_details_assigned_rider;

-- Step 1: Recreate old enum
CREATE TYPE delivery_status AS ENUM (
  'Pending',
  'Assigned',
  'RiderConfirmed',
  'EnRouteToPickup',
  'PickedUp',
  'EnRouteToDropoff',
  'Delivered',
  'Cancelled'
);

-- Step 2: Drop defaults
ALTER TABLE delivery_details ALTER COLUMN status DROP DEFAULT;
ALTER TABLE ride_session ALTER COLUMN status DROP DEFAULT;

-- Step 3: Convert enum -> TEXT
ALTER TABLE delivery_details
ALTER COLUMN status TYPE TEXT;

ALTER TABLE ride_session
ALTER COLUMN status TYPE TEXT;

-- Step 4: Convert TEXT -> old enum
ALTER TABLE delivery_details
ALTER COLUMN status TYPE delivery_status
  USING status::delivery_status;

ALTER TABLE ride_session
ALTER COLUMN status TYPE delivery_status
  USING status::delivery_status;

-- Step 5: Restore defaults
ALTER TABLE delivery_details
    ALTER COLUMN status SET DEFAULT 'Pending'::delivery_status;

ALTER TABLE ride_session
    ALTER COLUMN status SET DEFAULT 'Pending'::delivery_status;

-- Step 6: Recreate partial index with old enum
CREATE INDEX idx_delivery_details_assigned_rider
    ON delivery_details (assigned_rider_id)
    WHERE status = 'Assigned'::delivery_status;

-- Step 7: Drop new enum
DROP TYPE trip_status;