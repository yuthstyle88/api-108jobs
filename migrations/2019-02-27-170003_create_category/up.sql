CREATE TABLE category (
    id serial PRIMARY KEY,
    name varchar(20) NOT NULL UNIQUE,
    title varchar(100) NOT NULL,
    description text,
    category_id int REFERENCES category ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    creator_id int REFERENCES user_ ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    removed boolean DEFAULT FALSE NOT NULL,
    published timestamp NOT NULL DEFAULT now(),
    updated timestamp
);

CREATE TABLE category_moderator (
    id serial PRIMARY KEY,
    category_id int REFERENCES category ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    user_id int REFERENCES user_ ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    published timestamp NOT NULL DEFAULT now(),
    UNIQUE (category_id, user_id)
);

CREATE TABLE category_follower (
    id serial PRIMARY KEY,
    category_id int REFERENCES category ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    user_id int REFERENCES user_ ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    published timestamp NOT NULL DEFAULT now(),
    UNIQUE (category_id, user_id)
);

CREATE TABLE category_user_ban (
    id serial PRIMARY KEY,
    category_id int REFERENCES category ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    user_id int REFERENCES user_ ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    published timestamp NOT NULL DEFAULT now(),
    UNIQUE (category_id, user_id)
);

INSERT INTO category (name, title, category_id, creator_id)
    VALUES ('main', 'The Default Category', 1, 1);

CREATE TABLE site (
    id serial PRIMARY KEY,
    name varchar(20) NOT NULL UNIQUE,
    description text,
    creator_id int REFERENCES user_ ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    published timestamp NOT NULL DEFAULT now(),
    updated timestamp
);

