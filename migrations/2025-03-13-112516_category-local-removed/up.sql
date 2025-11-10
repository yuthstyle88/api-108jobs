-- Same for remote category, local removal should not be overwritten by
-- remove+restore on home instance
ALTER TABLE category
    ADD COLUMN local_removed boolean NOT NULL DEFAULT FALSE;

