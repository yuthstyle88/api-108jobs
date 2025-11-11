-- Add indexes on published column (needed for hot_rank updates)
CREATE INDEX idx_category_aggregates_published ON category_aggregates (published DESC);

CREATE INDEX idx_comment_aggregates_published ON comment_aggregates (published DESC);

