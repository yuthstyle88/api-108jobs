-- Restore the original column definition (no default).
ALTER TABLE tag
    ALTER COLUMN ap_id DROP DEFAULT;
