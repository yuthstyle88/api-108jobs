CREATE TABLE contact
(
    id              serial PRIMARY KEY,
    phone           text,
    email           text,
    secondary_email text,
    line_id         text,
    facebook        text,
    updated_at      timestamptz
);