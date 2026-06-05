--  Add the column back
ALTER TABLE category
    ADD COLUMN creator_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE;

-- Recreate the index
CREATE INDEX idx_category_creator ON category (creator_id);
