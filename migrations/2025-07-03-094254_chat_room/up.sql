CREATE TABLE chat_room
(
    id         serial PRIMARY KEY,
    post_id    int REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    created_at timestamp                                               NOT NULL DEFAULT now(),
    updated_at timestamp                                               NOT NULL
);