ALTER TABLE community
    ADD COLUMN path      ltree NOT NULL DEFAULT '0',
    ADD COLUMN slug      text         NOT NULL UNIQUE,
    ADD COLUMN active    boolean      NOT NULL DEFAULT true,
    ADD COLUMN is_new    boolean      NOT NULL DEFAULT true;

CREATE INDEX idx_community_path ON community USING GIST (path);
CREATE INDEX idx_community_slug_unique ON community (slug);
