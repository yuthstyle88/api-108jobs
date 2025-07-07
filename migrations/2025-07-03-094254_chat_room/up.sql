CREATE TABLE chat_room
(
    id         varchar PRIMARY KEY,
    room_name   VARCHAR,
    created_at timestamptz                                             NOT NULL DEFAULT now(),
    updated_at timestamptz                                             NOT NULL
);
