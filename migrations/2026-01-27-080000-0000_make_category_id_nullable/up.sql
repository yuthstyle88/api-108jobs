-- Make category_id nullable for posts
-- This allows delivery posts to exist without a category, relying on post_kind for distinction
ALTER TABLE post ALTER COLUMN category_id DROP NOT NULL;
