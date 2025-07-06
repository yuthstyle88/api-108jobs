-- Your SQL goes here
ALTER TABLE local_user
ALTER COLUMN public_key TYPE TEXT,
    ALTER COLUMN public_key DROP NOT NULL;
