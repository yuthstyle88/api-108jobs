CREATE TABLE user_key_mappings
(
    id              serial PRIMARY KEY,
    local_user_id   int  NOT NULL REFERENCES local_user
        ON UPDATE CASCADE ON DELETE CASCADE,
    hashed_password text NOT NULL, -- Hashed user password (bcrypt)
    real_key        text NOT NULL, -- Fixed KEK (256-bit, hex, encrypted server-side)
    created_at      timestamptz  NOT NULL DEFAULT now(),
    updated_at      timestamptz,
    UNIQUE (local_user_id)
);