CREATE TABLE identity_card
(
    id            serial PRIMARY KEY,
    address_id    int         REFERENCES address (id) ON DELETE SET NULL,
    id_number     text        NOT NULL,
    issued_date   date,
    expiry_date   date,
    full_name     text        NOT NULL,
    date_of_birth date,
    nationality   text,
    is_verified   boolean              DEFAULT FALSE,
    created_at    timestamptz NOT NULL DEFAULT now(),
    updated_at    timestamptz
);

CREATE INDEX idx_identity_card_id_number
    ON identity_card (id_number);
