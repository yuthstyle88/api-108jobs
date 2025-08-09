ALTER TABLE person
    ADD COLUMN identity_card_id INTEGER NOT NULL UNIQUE
        REFERENCES identity_card(id) ON DELETE RESTRICT;