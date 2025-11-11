ALTER TABLE category
    ADD COLUMN path      ltree NOT NULL DEFAULT '0',
    ADD COLUMN active    boolean      NOT NULL DEFAULT true,
    ADD COLUMN is_new    boolean      NOT NULL DEFAULT true;

CREATE INDEX idx_category_path ON category USING GIST (path);
