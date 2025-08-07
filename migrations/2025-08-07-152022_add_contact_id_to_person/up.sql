-- Your SQL goes here
ALTER TABLE person
ADD COLUMN contact_id INTEGER REFERENCES contact(id) ON DELETE SET NULL;