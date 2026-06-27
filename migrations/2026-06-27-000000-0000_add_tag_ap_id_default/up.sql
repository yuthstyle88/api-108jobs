-- The `tag` table was created (migration 2024-12-17-144959_category-post-tags)
-- with `ap_id text NOT NULL UNIQUE` but no DEFAULT, unlike every other
-- federatable table (post, category, person, …) whose `ap_id` defaults to
-- `generate_unique_changeme()`. Because `TagInsertForm` does not set `ap_id`,
-- every `Tag::create` — in tests and in production — fails with
-- `null value in column "ap_id" violates not-null constraint`.
--
-- Give `tag.ap_id` the same sequence-backed default the other tables use so
-- inserts that omit `ap_id` receive a unique placeholder. Additive and
-- non-destructive: NOT NULL and UNIQUE are preserved, existing rows untouched.
ALTER TABLE tag
    ALTER COLUMN ap_id SET DEFAULT generate_unique_changeme ();
