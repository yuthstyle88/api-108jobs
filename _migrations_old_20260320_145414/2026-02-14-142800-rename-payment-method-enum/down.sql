-- Revert payment_method enum values back to lowercase labels
-- From 'Cash'/'Coin' to 'cash'/'coin'
-- Safe to run multiple times due to IF EXISTS checks

DO $$
BEGIN
  -- Rename 'Cash' -> 'cash' if the current label exists
  IF EXISTS (
    SELECT 1 FROM pg_type t
    JOIN pg_enum e ON e.enumtypid = t.oid
    WHERE t.typname = 'payment_method' AND e.enumlabel = 'Cash'
  ) THEN
    ALTER TYPE payment_method RENAME VALUE 'Cash' TO 'cash';
  END IF;

  -- Rename 'Coin' -> 'coin' if the current label exists
  IF EXISTS (
    SELECT 1 FROM pg_type t
    JOIN pg_enum e ON e.enumtypid = t.oid
    WHERE t.typname = 'payment_method' AND e.enumlabel = 'Coin'
  ) THEN
    ALTER TYPE payment_method RENAME VALUE 'Coin' TO 'coin';
  END IF;
END $$;
