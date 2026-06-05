-- Add new enum value 'RideTaxi' to post_kind if it doesn't exist
DO $$
BEGIN
  IF NOT EXISTS (
    SELECT 1 FROM pg_type t
    JOIN pg_enum e ON t.oid = e.enumtypid
    WHERE t.typname = 'post_kind' AND e.enumlabel = 'RideTaxi'
  ) THEN
    ALTER TYPE post_kind ADD VALUE 'RideTaxi';
  END IF;
END$$;
