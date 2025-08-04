DO $$ BEGIN
CREATE TYPE role AS ENUM ('Employer', 'Freelancer', 'Admin');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

ALTER TABLE local_user
    ADD COLUMN IF NOT EXISTS role role NOT NULL DEFAULT 'Employer';
