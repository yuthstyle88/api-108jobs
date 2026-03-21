-- Add the admin_purge tables
CREATE TABLE admin_purge_person (
    id serial PRIMARY KEY,
    admin_person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    reason text,
    when_ timestamp NOT NULL DEFAULT now()
);

CREATE TABLE admin_purge_category (
    id serial PRIMARY KEY,
    admin_person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    reason text,
    when_ timestamp NOT NULL DEFAULT now()
);

CREATE TABLE admin_purge_post (
    id serial PRIMARY KEY,
    admin_person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    category_id int REFERENCES category ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    reason text,
    when_ timestamp NOT NULL DEFAULT now()
);

CREATE TABLE admin_purge_comment (
    id serial PRIMARY KEY,
    admin_person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    post_id int REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    reason text,
    when_ timestamp NOT NULL DEFAULT now()
);

