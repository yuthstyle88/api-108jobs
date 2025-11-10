-- these indices are used for single-category filtering and for the followed-communities (front-page) query
-- basically one index per Sort
-- index name is truncated to 63 chars so abbreviate a bit
CREATE INDEX idx_post_aggregates_category_active ON post_aggregates (category_id, featured_local DESC, hot_rank_active DESC, published DESC);

CREATE INDEX idx_post_aggregates_category_controversy ON post_aggregates (category_id, featured_local DESC, controversy_rank DESC);

CREATE INDEX idx_post_aggregates_category_hot ON post_aggregates (category_id, featured_local DESC, hot_rank DESC, published DESC);

CREATE INDEX idx_post_aggregates_category_scaled ON post_aggregates (category_id, featured_local DESC, scaled_rank DESC, published DESC);

CREATE INDEX idx_post_aggregates_category_most_comments ON post_aggregates (category_id, featured_local DESC, comments DESC, published DESC);

CREATE INDEX idx_post_aggregates_category_newest_comment_time ON post_aggregates (category_id, featured_local DESC, newest_comment_time DESC);

CREATE INDEX idx_post_aggregates_category_newest_comment_time_necro ON post_aggregates (category_id, featured_local DESC, newest_comment_time_necro DESC);

CREATE INDEX idx_post_aggregates_category_published ON post_aggregates (category_id, featured_local DESC, published DESC);

CREATE INDEX idx_post_aggregates_category_score ON post_aggregates (category_id, featured_local DESC, score DESC, published DESC);

-- these indices are used for "per-category" filtering
-- these indices weren't really useful because whenever the query filters by featured_category it also filters by category_id, so prepend that to all these indexes
DROP INDEX idx_post_aggregates_featured_category_active;

DROP INDEX idx_post_aggregates_featured_category_controversy;

DROP INDEX idx_post_aggregates_featured_category_hot;

DROP INDEX idx_post_aggregates_featured_category_scaled;

DROP INDEX idx_post_aggregates_featured_category_most_comments;

DROP INDEX idx_post_aggregates_featured_category_newest_comment_time;

DROP INDEX idx_post_aggregates_featured_category_newest_comment_time_necro;

DROP INDEX idx_post_aggregates_featured_category_published;

DROP INDEX idx_post_aggregates_featured_category_score;

CREATE INDEX idx_post_aggregates_featured_category_active ON post_aggregates (category_id, featured_category DESC, hot_rank_active DESC, published DESC);

CREATE INDEX idx_post_aggregates_featured_category_controversy ON post_aggregates (category_id, featured_category DESC, controversy_rank DESC);

CREATE INDEX idx_post_aggregates_featured_category_hot ON post_aggregates (category_id, featured_category DESC, hot_rank DESC, published DESC);

CREATE INDEX idx_post_aggregates_featured_category_scaled ON post_aggregates (category_id, featured_category DESC, scaled_rank DESC, published DESC);

CREATE INDEX idx_post_aggregates_featured_category_most_comments ON post_aggregates (category_id, featured_category DESC, comments DESC, published DESC);

CREATE INDEX idx_post_aggregates_featured_category_newest_comment_time ON post_aggregates (category_id, featured_category DESC, newest_comment_time DESC);

CREATE INDEX idx_post_aggregates_featured_category_newest_comment_time_necro ON post_aggregates (category_id, featured_category DESC, newest_comment_time_necro DESC);

CREATE INDEX idx_post_aggregates_featured_category_published ON post_aggregates (category_id, featured_category DESC, published DESC);

CREATE INDEX idx_post_aggregates_featured_category_score ON post_aggregates (category_id, featured_category DESC, score DESC, published DESC);

