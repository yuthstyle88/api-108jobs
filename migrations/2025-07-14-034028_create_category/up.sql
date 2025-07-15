CREATE
EXTENSION IF NOT EXISTS ltree;
CREATE TABLE category
(
    id         serial PRIMARY KEY,
    group_id   int REFERENCES category_group ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    path       ltree                                                             NOT NULL,
    title      text                                                              NOT NULL,
    subtitle   text,
    slug       text                                                              NOT NULL,
    image      text,
    active     boolean                                                                    DEFAULT TRUE,
    is_new     boolean,
    sort_order int                                                               NOT NULL,
    created_at timestamptz                                                       NOT NULL DEFAULT now(),
    updated_at timestamptz                                                       NOT NULL,
    UNIQUE (group_id, slug),
    UNIQUE (group_id, path),
    UNIQUE (group_id, sort_order)
);

CREATE INDEX idx_category_path_gist ON category USING GIST (path);
CREATE INDEX idx_category_group ON category (group_id);