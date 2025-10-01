CREATE TABLE last_reads
(
    local_user_id    int     NOT NULL REFERENCES local_user
        ON UPDATE CASCADE ON DELETE CASCADE,
    room_id          varchar NOT NULL REFERENCES chat_room
        ON UPDATE CASCADE ON DELETE CASCADE,
    last_read_msg_id varchar
        REFERENCES chat_message (msg_ref_id)
            ON UPDATE CASCADE ON DELETE SET NULL
            DEFERRABLE INITIALLY DEFERRED
        CHECK (last_read_msg_id IS NULL OR char_length(last_read_msg_id) > 0),
    updated_at       timestamptz,
    PRIMARY KEY (local_user_id, room_id)
);

-- Optional: index for faster lookups by last_read_msg_id
CREATE INDEX IF NOT EXISTS idx_last_reads_last_read_msg_id
  ON last_reads (last_read_msg_id);