ALTER TABLE local_user
    ADD COLUMN accepted_terms boolean NOT NULL DEFAULT FALSE;
