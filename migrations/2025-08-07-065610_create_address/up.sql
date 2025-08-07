-- Your SQL goes here
CREATE TABLE address
(
    id            SERIAL PRIMARY KEY,
    address_line1 TEXT    NOT NULL,
    address_line2 TEXT,
    subdistrict   TEXT,
    district      TEXT    NOT NULL,
    province      TEXT    NOT NULL,
    postal_code   TEXT    NOT NULL,
    country_id       VARCHAR(2)      DEFAULT 'TH',
    is_default    BOOLEAN   DEFAULT FALSE,
    created_at    TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at    TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);