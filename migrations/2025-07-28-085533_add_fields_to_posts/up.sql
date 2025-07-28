CREATE TYPE intended_use_enum AS ENUM ('Business', 'Personal', 'Unknown');

ALTER TABLE post
    ADD COLUMN slug text NOT NULL,
  ADD COLUMN is_english_required boolean NOT NULL DEFAULT false,
  ADD COLUMN budget numeric NOT NULL DEFAULT 0,
  ADD COLUMN deadline timestamptz,
  ADD COLUMN intended_use intended_use_enum NOT NULL DEFAULT 'Unknown';
