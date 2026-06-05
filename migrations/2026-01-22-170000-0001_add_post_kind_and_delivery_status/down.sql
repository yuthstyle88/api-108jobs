-- down.sql
-- Reverses the addition of post_kind column and drops the enum types safely

-- 1. Safely remove the column (this should always succeed)
ALTER TABLE post
DROP COLUMN IF EXISTS post_kind;

-- 2. Try to drop delivery_status (more specific / likely used later)
DO $$
BEGIN
DROP TYPE delivery_status;
EXCEPTION
    WHEN dependent_objects_still_exist THEN
        -- silently ignore → type is still in use (e.g. in delivery_details.status)
        NULL;
WHEN undefined_object THEN
        -- already gone → fine
        NULL;
WHEN OTHERS THEN
        RAISE;
END $$;

-- 3. Try to drop post_kind
DO $$
BEGIN
DROP TYPE post_kind;
EXCEPTION
    WHEN dependent_objects_still_exist THEN
        NULL;
WHEN undefined_object THEN
        NULL;
WHEN OTHERS THEN
        RAISE;
END $$;

-- Optional: logging (visible in migration output or logs)
-- DO $$ BEGIN
--     RAISE NOTICE 'post_kind column dropped';
--     RAISE NOTICE 'Attempted to drop post_kind and delivery_status enums (skipped if still referenced)';
-- END $$;