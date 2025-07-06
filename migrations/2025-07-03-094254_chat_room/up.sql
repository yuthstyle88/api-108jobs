CREATE TABLE chat_room
(
    id         varchar PRIMARY KEY,
    post_id    int REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    created_at timestamptz                                             NOT NULL DEFAULT now(),
    updated_at timestamptz                                             NOT NULL
);

CREATE TABLE chat_room_member
(
    room_id varchar REFERENCES chat_room ON UPDATE CASCADE ON DELETE CASCADE,
    user_id int REFERENCES local_user ON UPDATE CASCADE ON DELETE CASCADE,
    PRIMARY KEY (room_id, user_id)
);

CREATE INDEX idx_chat_room_member_user_id ON chat_room_member (user_id);
CREATE INDEX idx_chat_room_member_room_id ON chat_room_member (room_id);
