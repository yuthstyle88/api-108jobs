DROP INDEX IF EXISTS idx_billing_room_id;

ALTER TABLE billing
DROP COLUMN IF EXISTS room_id;