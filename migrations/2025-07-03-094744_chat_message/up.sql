CREATE TABLE chat_message
(
    id         serial PRIMARY KEY,
    room_id    int REFERENCES chat_room ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    sender_id  int REFERENCES local_user ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    content    text                                                          NOT NULL,
    file_url   text,
    file_type  varchar(50),
    file_name  text,
    status     smallint                                                      NOT NULL DEFAULT 1,
    created_at timestamptz                                                     NOT NULL DEFAULT now(),
    updated_at timestamptz                                                     NOT NULL
);

CREATE INDEX idx_chat_message_id ON chat_message (id);
CREATE INDEX idx_chat_message_room_id ON chat_message (room_id);