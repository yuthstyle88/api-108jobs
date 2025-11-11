-- Change category.visibility to allow values:
-- ('Public', 'LocalOnlyPublic', 'LocalOnlyPrivate','Private', 'Hidden')
-- rename old enum and add new one
ALTER TYPE category_visibility RENAME TO category_visibility__;

CREATE TYPE category_visibility AS enum (
    'Public',
    'LocalOnlyPublic',
    'LocalOnly',
    'Private',
    'Unlisted'
);

-- drop default value and index which reference old enum
ALTER TABLE category
    ALTER COLUMN visibility DROP DEFAULT;

DROP INDEX idx_category_random_number;

-- change the column type
ALTER TABLE category
    ALTER COLUMN visibility TYPE category_visibility
    USING visibility::text::category_visibility;

-- add default and index back in
ALTER TABLE category
    ALTER COLUMN visibility SET DEFAULT 'Public';

CREATE INDEX idx_category_random_number ON category (random_number) INCLUDE (local, self_promotion)
WHERE
    NOT (deleted OR removed OR visibility = 'Private' OR visibility = 'Unlisted');

DROP TYPE category_visibility__ CASCADE;

ALTER TYPE category_visibility RENAME VALUE 'LocalOnly' TO 'LocalOnlyPrivate';

-- write hidden value to visibility column
UPDATE
    category
SET
    visibility = 'Unlisted'
WHERE
    hidden;

-- drop the old hidden column
ALTER TABLE category
    DROP COLUMN hidden;

-- change modlog tables
ALTER TABLE modlog_combined
    DROP COLUMN mod_hide_category_id;

DROP TABLE mod_hide_category;

CREATE TABLE mod_change_category_visibility (
    id serial PRIMARY KEY,
    category_id int REFERENCES category ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    mod_person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    published timestamptz NOT NULL DEFAULT now(),
    reason text,
    visibility category_visibility NOT NULL
);

ALTER TABLE modlog_combined
    ADD COLUMN mod_change_category_visibility_id int REFERENCES mod_change_category_visibility (id) ON UPDATE CASCADE ON DELETE CASCADE,
    ADD CONSTRAINT modlog_combined_check CHECK ((num_nonnulls (admin_allow_instance_id, admin_block_instance_id, admin_purge_comment_id, admin_purge_category_id, admin_purge_person_id, admin_purge_post_id, mod_add_id, mod_add_category_id, mod_ban_id, mod_ban_from_category_id, mod_feature_post_id, mod_change_category_visibility_id, mod_lock_post_id, mod_remove_comment_id, mod_remove_category_id, mod_remove_post_id, mod_transfer_category_id) = 1));

