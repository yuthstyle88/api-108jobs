ALTER TABLE post
DROP COLUMN IF EXISTS is_english_required,
  DROP COLUMN IF EXISTS budget,
  DROP COLUMN IF EXISTS deadline,
  DROP COLUMN IF EXISTS job_type,
  DROP COLUMN IF EXISTS intended_use,
  ADD COLUMN ap_id TEXT,
  ADD CONSTRAINT idx_post_ap_id UNIQUE (ap_id);

DROP TYPE IF EXISTS job_type_enum;
DROP TYPE IF EXISTS intended_use_enum;
