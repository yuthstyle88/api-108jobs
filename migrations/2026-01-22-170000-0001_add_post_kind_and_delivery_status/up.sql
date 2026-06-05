-- Enum for distinguishing post kind
DO $$ BEGIN
  CREATE TYPE post_kind AS ENUM ('Normal', 'Delivery');
EXCEPTION
  WHEN duplicate_object THEN NULL;
END $$;

-- Optional: delivery status workflow for delivery jobs
DO $$ BEGIN
  CREATE TYPE delivery_status AS ENUM (
    'Pending',
    'Assigned',
    'EnRouteToPickup',
    'PickedUp',
    'EnRouteToDropoff',
    'Delivered',
    'Cancelled'
  );
EXCEPTION
  WHEN duplicate_object THEN NULL;
END $$;

-- Add column to post
ALTER TABLE post
  ADD COLUMN IF NOT EXISTS post_kind post_kind NOT NULL DEFAULT 'Normal';
