DO $$
BEGIN
  -- 1️⃣ Create old enum
  CREATE TYPE category_visibility_old AS ENUM (
    'Public',
    'LocalOnlyPublic',
    'LocalOnlyPrivate',
    'Private',
    'Hidden'
  );

  -- 2️⃣ Drop default FIRST
  ALTER TABLE category
    ALTER COLUMN visibility DROP DEFAULT;

  -- 2.1️⃣ Drop dependent index before type change to avoid enum operator mismatch
  --     (was recreated in the up.sql after type change)
  DROP INDEX IF EXISTS idx_category_random_number;

  -- 2.2️⃣ Remove objects introduced in up.sql that depend on category_visibility
  --       so we can safely drop the enum type later.
  --       (modlog_combined added a column referencing the new table, and
  --        mod_change_category_visibility.visibility uses category_visibility)
  ALTER TABLE modlog_combined
    DROP CONSTRAINT IF EXISTS modlog_combined_check,
    DROP COLUMN IF EXISTS mod_change_category_visibility_id;

  DROP TABLE IF EXISTS mod_change_category_visibility;

  -- 2.3️⃣ Restore legacy structures removed by up.sql so earlier migrations can run
  --       Recreate mod_hide_category (post-2024-12-08 schema uses published)
  CREATE TABLE IF NOT EXISTS mod_hide_category (
    id serial PRIMARY KEY,
    category_id int REFERENCES category ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    mod_person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    published timestamptz NOT NULL DEFAULT now(),
    reason text,
    hidden boolean DEFAULT FALSE
  );

  -- Add back the mod_hide_category reference on modlog_combined
  ALTER TABLE modlog_combined
    ADD COLUMN IF NOT EXISTS mod_hide_category_id int UNIQUE
      REFERENCES mod_hide_category (id)
      ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE;

  -- Re-add the single-nonnull check constraint matching the pre-2025-03-11 state
  ALTER TABLE modlog_combined
    ADD CONSTRAINT modlog_combined_check CHECK (
      num_nonnulls (
        admin_allow_instance_id,
        admin_block_instance_id,
        admin_purge_comment_id,
        admin_purge_category_id,
        admin_purge_person_id,
        admin_purge_post_id,
        mod_add_id,
        mod_add_category_id,
        mod_ban_id,
        mod_ban_from_category_id,
        mod_feature_post_id,
        mod_hide_category_id,
        mod_lock_post_id,
        mod_remove_comment_id,
        mod_remove_category_id,
        mod_remove_post_id,
        mod_transfer_category_id
      ) = 1
    );

  -- 3️⃣ Convert column safely
  ALTER TABLE category
    ALTER COLUMN visibility TYPE category_visibility_old
    USING (
      CASE
        WHEN visibility::text = 'Unlisted' THEN 'Hidden'
        ELSE visibility::text
      END
    )::category_visibility_old;

  -- 4️⃣ Drop new enum
  DROP TYPE category_visibility;

  -- 5️⃣ Rename old enum
  ALTER TYPE category_visibility_old RENAME TO category_visibility;

  -- 6️⃣ Restore default
  ALTER TABLE category
    ALTER COLUMN visibility SET DEFAULT 'Public';

  -- 6.5️⃣ Restore legacy 'hidden' column expected by earlier migrations
  ALTER TABLE category
    ADD COLUMN IF NOT EXISTS hidden boolean DEFAULT FALSE;

  -- Backfill from visibility semantics after enum restore (Hidden = true)
  UPDATE category SET hidden = (visibility = 'Hidden');

  -- 7️⃣ Recreate the index with the old semantics (Hidden instead of Unlisted)
  CREATE INDEX IF NOT EXISTS idx_category_random_number ON category (random_number)
  INCLUDE (local, self_promotion)
  WHERE NOT (deleted OR removed OR visibility = 'Private' OR visibility = 'Hidden');

END$$;