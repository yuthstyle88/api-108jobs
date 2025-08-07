-- Your SQL goes here
CREATE TABLE address
(
    id            SERIAL PRIMARY KEY,
    local_user_id INTEGER NOT NULL REFERENCES local_user (id) ON DELETE CASCADE,
    address_line1 TEXT    NOT NULL,
    address_line2 TEXT,
    subdistrict   TEXT    NOT NULL,
    district      TEXT    NOT NULL,
    province      TEXT    NOT NULL,
    postal_code   TEXT    NOT NULL,
    country       TEXT      DEFAULT 'Thailand',
    is_default    BOOLEAN   DEFAULT FALSE,
    created_at    TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at    TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);