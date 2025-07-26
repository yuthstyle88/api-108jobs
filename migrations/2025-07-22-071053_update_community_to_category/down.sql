DROP INDEX IF EXISTS idx_community_slug_unique;
DROP INDEX IF EXISTS idx_community_path;

ALTER TABLE community
DROP
COLUMN IF EXISTS path,
    DROP
COLUMN IF EXISTS slug,
    DROP
COLUMN IF EXISTS active,
    DROP
COLUMN IF EXISTS is_new;
