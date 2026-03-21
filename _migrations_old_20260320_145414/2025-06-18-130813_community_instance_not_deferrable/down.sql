-- Make constraint deferrable again (if exists)
DO $$
BEGIN
  ALTER TABLE category
    ALTER CONSTRAINT category_instance_id_fkey DEFERRABLE INITIALLY DEFERRED;
EXCEPTION
  WHEN undefined_object THEN
    -- Constraint doesn't exist, ignore
    NULL;
END $$;
