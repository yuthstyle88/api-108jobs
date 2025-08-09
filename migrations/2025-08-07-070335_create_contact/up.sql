CREATE TABLE contact
(
    id              serial PRIMARY KEY,
    phone           text,
    email           text,
    secondary_email text,
    line_id         text,
    facebook        text,
    created_at      timestamptz NOT NULL DEFAULT now(),
    updated_at      timestamptz
);