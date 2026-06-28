-- Drop pure federation tables
DROP TABLE IF EXISTS sent_activity;
DROP TABLE IF EXISTS received_activity;

-- person: drop federation columns (private_key and shared_key are kept for OTP/KYC)
ALTER TABLE person DROP COLUMN IF EXISTS ap_id;
ALTER TABLE person DROP COLUMN IF EXISTS local;
ALTER TABLE person DROP COLUMN IF EXISTS inbox_url;

-- post: drop federation columns
ALTER TABLE post DROP COLUMN IF EXISTS ap_id;
ALTER TABLE post DROP COLUMN IF EXISTS local;

-- comment: drop federation columns
ALTER TABLE comment DROP COLUMN IF EXISTS ap_id;
ALTER TABLE comment DROP COLUMN IF EXISTS local;

-- category: drop federation columns
ALTER TABLE category DROP COLUMN IF EXISTS ap_id;
ALTER TABLE category DROP COLUMN IF EXISTS local;
ALTER TABLE category DROP COLUMN IF EXISTS followers_url;
ALTER TABLE category DROP COLUMN IF EXISTS inbox_url;
ALTER TABLE category DROP COLUMN IF EXISTS moderators_url;
ALTER TABLE category DROP COLUMN IF EXISTS featured_url;

-- site: drop federation columns
ALTER TABLE site DROP COLUMN IF EXISTS ap_id;
ALTER TABLE site DROP COLUMN IF EXISTS inbox_url;
ALTER TABLE site DROP COLUMN IF EXISTS private_key;
ALTER TABLE site DROP COLUMN IF EXISTS public_key;

-- Drop federation-only enum type (was used only by sent_activity, now dropped)
DROP TYPE IF EXISTS actor_type_enum;

-- Add unique constraint on person name (replaces federation ap_id uniqueness)
CREATE UNIQUE INDEX IF NOT EXISTS idx_person_lower_name_unique ON person (lower(name));
