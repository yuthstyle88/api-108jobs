ALTER TABLE post
DROP COLUMN IF EXISTS is_english_required,
  DROP COLUMN IF EXISTS budget,
  DROP COLUMN IF EXISTS deadline,
  DROP COLUMN IF EXISTS job_type,
  DROP COLUMN IF EXISTS intended_use,

DROP TYPE IF EXISTS job_type_enum;
DROP TYPE IF EXISTS intended_use_enum;
