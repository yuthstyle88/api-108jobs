ALTER TABLE community
    ADD COLUMN path      ltree NOT NULL DEFAULT '0',
    ADD COLUMN active    boolean      NOT NULL DEFAULT true,
    ADD COLUMN is_new    boolean      NOT NULL DEFAULT true;

CREATE INDEX idx_community_path ON community USING GIST (path);
