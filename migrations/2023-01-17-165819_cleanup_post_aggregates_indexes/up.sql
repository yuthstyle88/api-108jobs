-- Drop the old indexes
DROP INDEX idx_post_aggregates_newest_comment_time, idx_post_aggregates_comments, idx_post_aggregates_hot, idx_post_aggregates_active, idx_post_aggregates_score, idx_post_aggregates_published;

-- All of the post fetching queries now start with either
-- featured_local desc, or featured_category desc, then the other sorts.
-- So you now need to double these indexes
CREATE INDEX idx_post_aggregates_featured_local_newest_comment_time ON post_aggregates (featured_local DESC, newest_comment_time DESC);

CREATE INDEX idx_post_aggregates_featured_category_newest_comment_time ON post_aggregates (featured_category DESC, newest_comment_time DESC);

CREATE INDEX idx_post_aggregates_featured_local_comments ON post_aggregates (featured_local DESC, comments DESC);

CREATE INDEX idx_post_aggregates_featured_category_comments ON post_aggregates (featured_category DESC, comments DESC);

CREATE INDEX idx_post_aggregates_featured_local_hot ON post_aggregates (featured_local DESC, hot_rank (score, published) DESC, published DESC);

CREATE INDEX idx_post_aggregates_featured_category_hot ON post_aggregates (featured_category DESC, hot_rank (score, published) DESC, published DESC);

CREATE INDEX idx_post_aggregates_featured_local_active ON post_aggregates (featured_local DESC, hot_rank (score, newest_comment_time) DESC, newest_comment_time DESC);

CREATE INDEX idx_post_aggregates_featured_category_active ON post_aggregates (featured_category DESC, hot_rank (score, newest_comment_time) DESC, newest_comment_time DESC);

CREATE INDEX idx_post_aggregates_featured_local_score ON post_aggregates (featured_local DESC, score DESC);

CREATE INDEX idx_post_aggregates_featured_category_score ON post_aggregates (featured_category DESC, score DESC);

CREATE INDEX idx_post_aggregates_featured_local_published ON post_aggregates (featured_local DESC, published DESC);

CREATE INDEX idx_post_aggregates_featured_category_published ON post_aggregates (featured_category DESC, published DESC);

