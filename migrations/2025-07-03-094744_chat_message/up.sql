CREATE TABLE chat_message
(
    id         serial PRIMARY KEY,
    room_id    varchar     NOT NULL REFERENCES chat_room
        ON UPDATE CASCADE ON DELETE CASCADE,
    sender_id  int         NOT NULL REFERENCES local_user
        ON UPDATE CASCADE ON DELETE CASCADE,
    content    text        NOT NULL,
    status     smallint    NOT NULL DEFAULT 1,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz
);

CREATE INDEX idx_chat_message_id ON chat_message (id);
CREATE INDEX idx_chat_message_room_id ON chat_message (room_id);
