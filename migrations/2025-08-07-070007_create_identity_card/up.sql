CREATE TABLE identity_card
(
    id            serial PRIMARY KEY,
    address_id int NOT NULL REFERENCES address(id) ON DELETE RESTRICT,
    id_number     text        NOT NULL,
    issued_date   date,
    expiry_date   date,
    full_name     text        NOT NULL,
    date_of_birth date,
    nationality   text,
    is_verified   boolean              DEFAULT FALSE,
    updated_at    timestamptz
);

CREATE INDEX idx_identity_card_id_number
    ON identity_card (id_number);
