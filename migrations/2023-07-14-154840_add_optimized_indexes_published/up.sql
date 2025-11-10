-- Drop the old indexes
DROP INDEX idx_post_aggregates_featured_local_hot;

DROP INDEX idx_post_aggregates_featured_local_active;

DROP INDEX idx_post_aggregates_featured_local_score;

DROP INDEX idx_post_aggregates_featured_category_hot;

DROP INDEX idx_post_aggregates_featured_category_active;

DROP INDEX idx_post_aggregates_featured_category_score;

DROP INDEX idx_comment_aggregates_hot;

DROP INDEX idx_comment_aggregates_score;

-- Add a published desc, to the end of the hot and active ranks
-- Add missing most comments index
CREATE INDEX idx_post_aggregates_featured_local_most_comments ON post_aggregates (featured_local DESC, comments DESC, published DESC);

CREATE INDEX idx_post_aggregates_featured_category_most_comments ON post_aggregates (featured_category DESC, comments DESC, published DESC);

-- featured_local
CREATE INDEX idx_post_aggregates_featured_local_hot ON post_aggregates (featured_local DESC, hot_rank DESC, published DESC);

CREATE INDEX idx_post_aggregates_featured_local_active ON post_aggregates (featured_local DESC, hot_rank_active DESC, published DESC);

CREATE INDEX idx_post_aggregates_featured_local_score ON post_aggregates (featured_local DESC, score DESC, published DESC);

-- featured_category
CREATE INDEX idx_post_aggregates_featured_category_hot ON post_aggregates (featured_category DESC, hot_rank DESC, published DESC);

CREATE INDEX idx_post_aggregates_featured_category_active ON post_aggregates (featured_category DESC, hot_rank_active DESC, published DESC);

CREATE INDEX idx_post_aggregates_featured_category_score ON post_aggregates (featured_category DESC, score DESC, published DESC);

-- Fixing some comment aggregates ones
CREATE INDEX idx_comment_aggregates_hot ON comment_aggregates (hot_rank DESC, published DESC);

CREATE INDEX idx_comment_aggregates_score ON comment_aggregates (score DESC, published DESC);

