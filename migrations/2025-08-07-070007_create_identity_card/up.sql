-- Your SQL goes here

CREATE TABLE identity_card
(
    id            SERIAL PRIMARY KEY,
    local_user_id INTEGER NOT NULL REFERENCES local_user (id) ON DELETE CASCADE,
    address_id    INTEGER REFERENCES address (id) ON DELETE SET NULL,
    id_number     TEXT    NOT NULL UNIQUE,
    issued_date   DATE,
    expiry_date   DATE,
    full_name     TEXT,
    date_of_birth DATE,
    nationality   TEXT,
    is_verified   BOOLEAN   DEFAULT FALSE,
    created_at    TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);