CREATE TYPE category_visibility AS enum (
    'Public',
    'LocalOnly'
);

ALTER TABLE category
    ADD COLUMN visibility category_visibility NOT NULL DEFAULT 'Public';

