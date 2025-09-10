CREATE TABLE chat_message
(
    message_id varchar     NOT NULL,
    room_id    varchar     NOT NULL REFERENCES chat_room
        ON UPDATE CASCADE ON DELETE CASCADE,
    sender_id  int         NOT NULL REFERENCES local_user
        ON UPDATE CASCADE ON DELETE CASCADE,
    content    text        NOT NULL,
    status     smallint    NOT NULL DEFAULT 1,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz,
    PRIMARY KEY (room_id, message_id)
);

CREATE INDEX idx_chat_message_sender_id ON chat_message (sender_id);
CREATE INDEX idx_chat_message_created_at ON chat_message (created_at);
