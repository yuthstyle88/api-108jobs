-- Add the blur_self_promotion to the local user table as a setting
ALTER TABLE local_user
    ADD COLUMN blur_self_promotion boolean NOT NULL DEFAULT TRUE;

-- Add the auto_expand to the local user table as a setting
ALTER TABLE local_user
    ADD COLUMN auto_expand boolean NOT NULL DEFAULT FALSE;

