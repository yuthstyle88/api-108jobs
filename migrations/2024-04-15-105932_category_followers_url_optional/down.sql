-- First set unique values for null columns (generate unique followers_url based on id)
UPDATE category SET followers_url = 'https://example.com/category/' || id::text || '/followers' WHERE followers_url IS NULL;

-- Then set NOT NULL
ALTER TABLE category
    ALTER COLUMN followers_url SET NOT NULL;
