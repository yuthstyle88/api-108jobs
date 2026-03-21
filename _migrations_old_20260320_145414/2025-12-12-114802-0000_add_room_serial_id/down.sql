-- Drop the unique constraint
ALTER TABLE chat_room DROP CONSTRAINT IF EXISTS chat_room_serial_id_unique;

-- Drop the column
ALTER TABLE chat_room DROP COLUMN IF EXISTS serial_id;

-- Drop the sequence created by BIGSERIAL (Postgres auto-creates it)
DROP SEQUENCE IF EXISTS chat_room_serial_id_seq;
