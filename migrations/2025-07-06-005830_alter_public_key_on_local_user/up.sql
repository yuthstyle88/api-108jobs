-- Your SQL goes here
ALTER TABLE local_user
    ADD COLUMN public_key TEXT;
ALTER TABLE local_user
    ADD COLUMN roles TEXT NOT NULL DEFAULT '[]';