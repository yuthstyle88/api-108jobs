-- NOTE: Up migration data loss is NOT recovered by this rollback. Structure only.

CREATE TYPE IF NOT EXISTS actor_type_enum AS ENUM ('Person', 'Category', 'Site');

CREATE TABLE IF NOT EXISTS received_activity (
  ap_id text PRIMARY KEY,
  published_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS sent_activity (
  id bigserial PRIMARY KEY,
  ap_id text NOT NULL UNIQUE,
  data json NOT NULL,
  sensitive bool NOT NULL DEFAULT false,
  published_at timestamptz NOT NULL DEFAULT now(),
  send_inboxes text[],
  send_category_followers_of int4,
  send_all_instances bool NOT NULL DEFAULT false,
  actor_type actor_type_enum NOT NULL,
  actor_apub_id text
);

DROP INDEX IF EXISTS idx_person_lower_name_unique;
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
