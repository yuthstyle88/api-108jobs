CREATE TABLE chat_room
(
    id         varchar PRIMARY KEY,
    room_name  varchar,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz
);

CREATE TABLE chat_participant
(
    room_id   varchar     NOT NULL REFERENCES chat_room (id) ON DELETE CASCADE,
    member_id int         NOT NULL REFERENCES local_user (id) ON DELETE CASCADE,
    joined_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (room_id, member_id)
);
