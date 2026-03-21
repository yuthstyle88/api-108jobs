ALTER TABLE mod_change_category_visibility
    ADD COLUMN reason text,
    ADD COLUMN visibility_new category_visibility;

UPDATE
    mod_change_category_visibility
SET
    visibility_new = visibility;

ALTER TABLE mod_change_category_visibility
    DROP COLUMN visibility;

ALTER TABLE mod_change_category_visibility RENAME COLUMN visibility_new TO visibility;

ALTER TABLE mod_change_category_visibility
    ALTER COLUMN visibility SET NOT NULL;

