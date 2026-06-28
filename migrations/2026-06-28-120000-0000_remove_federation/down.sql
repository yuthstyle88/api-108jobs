-- NOTE: Data dropped by up.sql is not recoverable.
-- This down.sql only restores table/column structure (columns will be empty/NULL).

-- Re-add pure federation tables (empty)
CREATE TABLE IF NOT EXISTS received_activity (
  ap_id text PRIMARY KEY,
  published timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS sent_activity (
  id serial4 PRIMARY KEY,
  ap_id text NOT NULL UNIQUE,
  data jsonb NOT NULL,
  sensitive bool NOT NULL DEFAULT false,
  published timestamptz NOT NULL DEFAULT now(),
  send_inboxes text[],
  send_category_followers_of int4,
  send_all_instances bool NOT NULL DEFAULT false,
  actor_type text NOT NULL,
  actor_apub_id text
);

-- Re-add federation columns (nullable to avoid constraint failures on empty rollback)
ALTER TABLE person ADD COLUMN IF NOT EXISTS ap_id text;
ALTER TABLE person ADD COLUMN IF NOT EXISTS local boolean DEFAULT true;
ALTER TABLE person ADD COLUMN IF NOT EXISTS inbox_url text;
ALTER TABLE post ADD COLUMN IF NOT EXISTS ap_id text;
ALTER TABLE post ADD COLUMN IF NOT EXISTS local boolean DEFAULT true;
ALTER TABLE comment ADD COLUMN IF NOT EXISTS ap_id text;
ALTER TABLE comment ADD COLUMN IF NOT EXISTS local boolean DEFAULT true;
ALTER TABLE category ADD COLUMN IF NOT EXISTS ap_id text;
ALTER TABLE category ADD COLUMN IF NOT EXISTS local boolean DEFAULT true;
ALTER TABLE category ADD COLUMN IF NOT EXISTS followers_url text;
ALTER TABLE category ADD COLUMN IF NOT EXISTS inbox_url text;
ALTER TABLE category ADD COLUMN IF NOT EXISTS moderators_url text;
ALTER TABLE category ADD COLUMN IF NOT EXISTS featured_url text;
ALTER TABLE site ADD COLUMN IF NOT EXISTS ap_id text;
ALTER TABLE site ADD COLUMN IF NOT EXISTS inbox_url text;
ALTER TABLE site ADD COLUMN IF NOT EXISTS private_key text;
ALTER TABLE site ADD COLUMN IF NOT EXISTS public_key text;
