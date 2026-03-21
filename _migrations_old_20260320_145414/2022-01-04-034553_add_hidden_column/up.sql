ALTER TABLE category
    ADD COLUMN hidden boolean DEFAULT FALSE;

CREATE TABLE mod_hide_category (
    id serial PRIMARY KEY,
    category_id int REFERENCES category ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    mod_person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    when_ timestamp NOT NULL DEFAULT now(),
    reason text,
    hidden boolean DEFAULT FALSE
);

