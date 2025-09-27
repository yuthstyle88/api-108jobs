CREATE TABLE last_reads
(
    user_id          int         NOT NULL REFERENCES local_user
        ON UPDATE CASCADE ON DELETE CASCADE,
    room_id          varchar     NOT NULL REFERENCES chat_room
        ON UPDATE CASCADE ON DELETE CASCADE,
    last_read_msg_id int         NOT NULL REFERENCES local_user
        ON UPDATE CASCADE ON DELETE CASCADE,
    updated_at       timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, room_id)
);