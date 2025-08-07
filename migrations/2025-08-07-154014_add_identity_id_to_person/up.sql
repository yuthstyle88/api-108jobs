-- Your SQL goes here
ALTER TABLE person
ADD COLUMN identity_card_id INTEGER REFERENCES identity_card(id) ON DELETE SET NULL;