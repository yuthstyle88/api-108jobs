CREATE TABLE chat_message
(
    id          BIGSERIAL PRIMARY KEY,
    msg_ref_id  varchar     NOT NULL,
    room_id     varchar     NOT NULL REFERENCES chat_room
        ON UPDATE CASCADE ON DELETE CASCADE,
    sender_id   int         REFERENCES local_user
                                ON UPDATE CASCADE ON DELETE SET NULL,
    receiver_id int         REFERENCES local_user
                                ON UPDATE CASCADE ON DELETE SET NULL,
    content     text        NOT NULL,
    status      smallint    NOT NULL DEFAULT 1,
    created_at  timestamptz NOT NULL DEFAULT now(),
    updated_at  timestamptz
);

-- ไม่ต้องสร้าง index ซ้ำกับ UNIQUE
ALTER TABLE chat_message
    ADD CONSTRAINT chat_message_msg_ref_id_unique UNIQUE (msg_ref_id);

CREATE INDEX idx_chat_message_room_id ON chat_message (room_id);