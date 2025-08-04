CREATE TYPE intended_use_enum AS ENUM ('Business', 'Personal', 'Unknown');
CREATE TYPE job_type_enum AS ENUM ('Freelance', 'Contract', 'PartTime', 'FullTime');

ALTER TABLE post
  ADD COLUMN is_english_required boolean NOT NULL DEFAULT false,
  ADD COLUMN budget double precision NOT NULL DEFAULT 0,
  ADD COLUMN deadline timestamptz,
  ADD COLUMN job_type job_type_enum NOT NULL DEFAULT 'PartTime',
  ADD COLUMN intended_use intended_use_enum NOT NULL DEFAULT 'Unknown';
