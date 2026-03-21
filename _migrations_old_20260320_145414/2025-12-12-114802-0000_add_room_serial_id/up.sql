ALTER TABLE chat_room
    ADD COLUMN serial_id BIGSERIAL;

ALTER TABLE chat_room
    ADD CONSTRAINT chat_room_serial_id_unique UNIQUE (serial_id);

-- Backfill existing rows (safe, fast on small tables; use a loop or batch for huge tables)
UPDATE chat_room
SET serial_id = nextval('chat_room_serial_id_seq')
WHERE serial_id IS NULL;