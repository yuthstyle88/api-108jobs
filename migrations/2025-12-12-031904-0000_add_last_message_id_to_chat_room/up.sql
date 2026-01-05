ALTER TABLE chat_room
    ADD COLUMN last_message_id VARCHAR CHECK (last_message_id IS NULL OR char_length(last_message_id) > 0),
    ADD COLUMN last_message_at TIMESTAMPTZ;
