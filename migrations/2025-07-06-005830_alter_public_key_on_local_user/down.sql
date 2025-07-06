-- This file should undo anything in `up.sql`
ALTER TABLE local_user
    ALTER COLUMN public_key SET NOT NULL;
