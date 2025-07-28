ALTER TABLE post
DROP COLUMN slug,
  DROP COLUMN is_english_required,
  DROP COLUMN budget,
  DROP COLUMN deadline,
  DROP COLUMN intended_use;

DROP TYPE intended_use_enum;
