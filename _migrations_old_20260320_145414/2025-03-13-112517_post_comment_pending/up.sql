-- When posting to a remote category mark it as pending until it gets announced back to us.
-- This way the posts of banned users wont appear in the category on other instances.
ALTER TABLE post
    ADD COLUMN federation_pending boolean NOT NULL DEFAULT FALSE;

ALTER TABLE comment
    ADD COLUMN federation_pending boolean NOT NULL DEFAULT FALSE;

