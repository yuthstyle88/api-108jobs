CREATE TABLE category_group
(
    id         serial PRIMARY KEY,
    title      text        NOT NULL,
    sort_order int NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL
);
