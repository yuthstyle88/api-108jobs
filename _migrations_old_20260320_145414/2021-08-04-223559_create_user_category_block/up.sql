CREATE TABLE person_block (
    id serial PRIMARY KEY,
    person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    target_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    published timestamp NOT NULL DEFAULT now(),
    UNIQUE (person_id, target_id)
);

CREATE TABLE category_block (
    id serial PRIMARY KEY,
    person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    category_id int REFERENCES category ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    published timestamp NOT NULL DEFAULT now(),
    UNIQUE (person_id, category_id)
);

