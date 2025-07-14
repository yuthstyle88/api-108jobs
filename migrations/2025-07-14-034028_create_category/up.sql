CREATE
EXTENSION IF NOT EXISTS ltree;
CREATE TABLE category
(
    id         serial PRIMARY KEY,
    path       ltree       NOT NULL,
    title      text        NOT NULL,
    slug       text        NOT NULL,
    image      text,
    active     boolean              DEFAULT TRUE,
    is_new     boolean,
    sort_order int         NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL
);

CREATE INDEX idx_category_path_gist ON category USING GIST (path);