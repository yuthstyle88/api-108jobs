DROP INDEX idx_post_aggregates_featured_category_active;

DROP INDEX idx_post_aggregates_featured_category_controversy;

DROP INDEX idx_post_aggregates_featured_category_hot;

DROP INDEX idx_post_aggregates_featured_category_scaled;

DROP INDEX idx_post_aggregates_featured_category_most_comments;

DROP INDEX idx_post_aggregates_featured_category_newest_comment_time;

DROP INDEX idx_post_aggregates_featured_category_newest_comment_time_necro;

DROP INDEX idx_post_aggregates_featured_category_published;

DROP INDEX idx_post_aggregates_featured_category_score;

CREATE INDEX idx_post_aggregates_featured_category_active ON post_aggregates (featured_category DESC, hot_rank_active DESC, published DESC);

CREATE INDEX idx_post_aggregates_featured_category_controversy ON post_aggregates (featured_category DESC, controversy_rank DESC);

CREATE INDEX idx_post_aggregates_featured_category_hot ON post_aggregates (featured_category DESC, hot_rank DESC, published DESC);

CREATE INDEX idx_post_aggregates_featured_category_scaled ON post_aggregates (featured_category DESC, scaled_rank DESC, published DESC);

CREATE INDEX idx_post_aggregates_featured_category_most_comments ON post_aggregates (featured_category DESC, comments DESC, published DESC);

CREATE INDEX idx_post_aggregates_featured_category_newest_comment_time ON post_aggregates (featured_category DESC, newest_comment_time DESC);

CREATE INDEX idx_post_aggregates_featured_category_newest_comment_time_necro ON post_aggregates (featured_category DESC, newest_comment_time_necro DESC);

CREATE INDEX idx_post_aggregates_featured_category_published ON post_aggregates (featured_category DESC, published DESC);

CREATE INDEX idx_post_aggregates_featured_category_score ON post_aggregates (featured_category DESC, score DESC, published DESC);

DROP INDEX idx_post_aggregates_category_active;

DROP INDEX idx_post_aggregates_category_controversy;

DROP INDEX idx_post_aggregates_category_hot;

DROP INDEX idx_post_aggregates_category_scaled;

DROP INDEX idx_post_aggregates_category_most_comments;

DROP INDEX idx_post_aggregates_category_newest_comment_time;

DROP INDEX idx_post_aggregates_category_newest_comment_time_necro;

DROP INDEX idx_post_aggregates_category_published;

DROP INDEX idx_post_aggregates_category_score;

