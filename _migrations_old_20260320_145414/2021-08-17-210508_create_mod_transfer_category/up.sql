-- Add the mod_transfer_category log table
CREATE TABLE mod_transfer_category (
    id serial PRIMARY KEY,
    mod_person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    other_person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    category_id int REFERENCES category ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    removed boolean DEFAULT FALSE,
    when_ timestamp NOT NULL DEFAULT now()
);

