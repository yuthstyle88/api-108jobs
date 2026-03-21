-- We should remove existing deferrable constraints, as they're potentially dangerous.
--
-- This is the only one I could find after doing a DB dump.
ALTER TABLE category
    ALTER CONSTRAINT category_instance_id_fkey NOT DEFERRABLE;

