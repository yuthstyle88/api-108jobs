-- Restore the pre-rename column name.
ALTER TABLE admin_allow_instance RENAME COLUMN published_at TO published;
