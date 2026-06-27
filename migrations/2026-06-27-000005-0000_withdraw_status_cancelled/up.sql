-- Add Cancelled to the withdraw_status enum.
-- Postgres cannot remove enum values, so the down migration is intentionally a no-op.
ALTER TYPE withdraw_status ADD VALUE IF NOT EXISTS 'Cancelled';
