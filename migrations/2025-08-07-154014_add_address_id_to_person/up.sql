-- Your SQL goes here
ALTER TABLE person
ADD COLUMN address_id INTEGER REFERENCES contact(id) ON DELETE SET NULL;