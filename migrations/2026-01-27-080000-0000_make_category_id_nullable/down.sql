-- Revert: make category_id required again
ALTER TABLE post ALTER COLUMN category_id SET NOT NULL;
