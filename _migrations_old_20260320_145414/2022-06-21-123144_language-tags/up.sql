CREATE TABLE
LANGUAGE (
    id integer PRIMARY KEY,
    code varchar(3),
    name text
);

CREATE TABLE local_user_language (
    id serial PRIMARY KEY,
    local_user_id int REFERENCES local_user ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    language_id int REFERENCES
    LANGUAGE ON
    UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    UNIQUE (local_user_id, language_id)
);

ALTER TABLE local_user RENAME COLUMN lang TO interface_language;

INSERT INTO
LANGUAGE (id, code, name)
    VALUES (0, 'und', 'Undetermined');

ALTER TABLE post
    ADD COLUMN language_id integer REFERENCES LANGUAGE NOT
    NULL DEFAULT 0;

INSERT INTO
LANGUAGE (id,code, name)
    VALUES (1,'en', 'English');

INSERT INTO
LANGUAGE (id, code, name)
    VALUES (66,'th', 'ไทย');

INSERT INTO
LANGUAGE (id, code, name)
    VALUES (84,'vi', 'Tiếng Việt');

