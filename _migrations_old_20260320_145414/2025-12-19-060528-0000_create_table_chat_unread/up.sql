CREATE TABLE chat_unread
(
    local_user_id   int         NOT NULL REFERENCES local_user (id) ON DELETE CASCADE,
    room_id         VARCHAR     NOT NULL REFERENCES chat_room (id)
        ON UPDATE CASCADE ON DELETE CASCADE,
    unread_count    INT         NOT NULL DEFAULT 0,
    last_message_id VARCHAR CHECK (last_message_id IS NULL OR char_length(last_message_id) > 0),
    last_message_at TIMESTAMPTZ,
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (local_user_id, room_id)
);