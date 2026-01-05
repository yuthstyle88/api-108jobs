--  Add the column back
ALTER TABLE category
    ADD COLUMN creator_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE;

-- Recreate the index
CREATE INDEX idx_category_creator ON category (creator_id);

-- Add the data, selecting the highest mod
UPDATE
    category
SET
    creator_id = sub.person_id
FROM (
    SELECT
        cm.category_id,
        cm.person_id
    FROM
        category_moderator cm
    LIMIT 1) AS sub
WHERE
    id = sub.category_id;

-- Set to not null
ALTER TABLE category
    ALTER COLUMN creator_id SET NOT NULL;

