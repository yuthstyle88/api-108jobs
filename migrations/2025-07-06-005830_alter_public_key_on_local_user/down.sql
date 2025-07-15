ALTER TABLE local_user
DROP COLUMN IF EXISTS public_key;

ALTER TABLE local_user
DROP COLUMN IF EXISTS role;

-- Drop the enum type only if it's not used elsewhere
DO $$
BEGIN
    -- Check if the type exists and is not used by any other table
    IF EXISTS (
        SELECT 1 FROM pg_type WHERE typname = 'role'
    ) THEN
        -- Safely drop enum type if nothing else depends on it
DROP TYPE role;
END IF;
END $$;
