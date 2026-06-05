-- Revert: add back hide_modlog_mod_names column
ALTER TABLE local_site
    ADD COLUMN IF NOT EXISTS hide_modlog_mod_names boolean DEFAULT TRUE NOT NULL;
