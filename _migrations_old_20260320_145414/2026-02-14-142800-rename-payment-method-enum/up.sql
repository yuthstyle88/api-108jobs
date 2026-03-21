-- Rename payment_method enum values to match Rust enum (DbValueStyle = "verbatim")
-- From 'cash'/'coin' to 'Cash'/'Coin'
-- Safe to run multiple times due to IF EXISTS checks

DO $$
BEGIN
  -- Rename 'cash' -> 'Cash' if the old label exists
  IF EXISTS (
    SELECT 1 FROM pg_type t
    JOIN pg_enum e ON e.enumtypid = t.oid
    WHERE t.typname = 'payment_method' AND e.enumlabel = 'cash'
  ) THEN
    ALTER TYPE payment_method RENAME VALUE 'cash' TO 'Cash';
  END IF;

  -- Rename 'coin' -> 'Coin' if the old label exists
  IF EXISTS (
    SELECT 1 FROM pg_type t
    JOIN pg_enum e ON e.enumtypid = t.oid
    WHERE t.typname = 'payment_method' AND e.enumlabel = 'coin'
  ) THEN
    ALTER TYPE payment_method RENAME VALUE 'coin' TO 'Coin';
  END IF;
END $$;
