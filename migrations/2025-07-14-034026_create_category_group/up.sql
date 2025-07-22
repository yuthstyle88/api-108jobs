CREATE TABLE category_group
(
    id         serial PRIMARY KEY,
    title      text        NOT NULL,
    active     boolean     NOT NULL DEFAULT true,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL
);
