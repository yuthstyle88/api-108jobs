-- Identity cards
CREATE TABLE identity_cards (
    id              SERIAL PRIMARY KEY,
    address_id      INTEGER,
    id_number       VARCHAR(64)  NOT NULL,
    issued_date     DATE         NOT NULL,
    expiry_date     DATE         NOT NULL,
    full_name       VARCHAR(255) NOT NULL,
    date_of_birth   DATE         NOT NULL,
    nationality     VARCHAR(255) NOT NULL,
    is_verified     BOOLEAN      NOT NULL DEFAULT FALSE,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ
);

-- Helpful indexes and constraints
CREATE INDEX idx_identity_cards_user ON identity_cards (local_user_id);
CREATE INDEX idx_identity_cards_verified ON identity_cards (is_verified);
CREATE UNIQUE INDEX uniq_identity_card_per_user ON identity_cards (local_user_id, id_number);
