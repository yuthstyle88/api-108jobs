-- This converts the old hot_rank functions, to columns
-- Remove the old compound indexes
DROP INDEX idx_post_aggregates_featured_local_newest_comment_time;

DROP INDEX idx_post_aggregates_featured_category_newest_comment_time;

DROP INDEX idx_post_aggregates_featured_local_comments;

DROP INDEX idx_post_aggregates_featured_category_comments;

DROP INDEX idx_post_aggregates_featured_local_hot;

DROP INDEX idx_post_aggregates_featured_category_hot;

DROP INDEX idx_post_aggregates_featured_local_score;

DROP INDEX idx_post_aggregates_featured_category_score;

DROP INDEX idx_post_aggregates_featured_local_published;

DROP INDEX idx_post_aggregates_featured_category_published;

DROP INDEX idx_post_aggregates_featured_local_active;

DROP INDEX idx_post_aggregates_featured_category_active;

DROP INDEX idx_comment_aggregates_hot;

DROP INDEX idx_category_aggregates_hot;

-- Add the new hot rank columns for post and comment aggregates
-- Note: 1728 is the result of the hot_rank function, with a score of 1, posted now
-- hot_rank = 10000*log10(1 + 3)/Power(2, 1.8)
ALTER TABLE post_aggregates
    ADD COLUMN hot_rank integer NOT NULL DEFAULT 1728;

ALTER TABLE post_aggregates
    ADD COLUMN hot_rank_active integer NOT NULL DEFAULT 1728;

ALTER TABLE comment_aggregates
    ADD COLUMN hot_rank integer NOT NULL DEFAULT 1728;

ALTER TABLE category_aggregates
    ADD COLUMN hot_rank integer NOT NULL DEFAULT 1728;

-- Populate them initially
-- Note: After initial population, these are updated in a periodic scheduled job,
-- with only the last week being updated.
UPDATE
    post_aggregates
SET
    hot_rank_active = hot_rank (score::numeric, newest_comment_time_necro);

UPDATE
    post_aggregates
SET
    hot_rank = hot_rank (score::numeric, published);

UPDATE
    comment_aggregates
SET
    hot_rank = hot_rank (score::numeric, published);

UPDATE
    category_aggregates
SET
    hot_rank = hot_rank (subscribers::numeric, published);

-- Create single column indexes
CREATE INDEX idx_post_aggregates_score ON post_aggregates (score DESC);

CREATE INDEX idx_post_aggregates_published ON post_aggregates (published DESC);

CREATE INDEX idx_post_aggregates_newest_comment_time ON post_aggregates (newest_comment_time DESC);

CREATE INDEX idx_post_aggregates_newest_comment_time_necro ON post_aggregates (newest_comment_time_necro DESC);

CREATE INDEX idx_post_aggregates_featured_category ON post_aggregates (featured_category DESC);

CREATE INDEX idx_post_aggregates_featured_local ON post_aggregates (featured_local DESC);

CREATE INDEX idx_post_aggregates_hot ON post_aggregates (hot_rank DESC);

CREATE INDEX idx_post_aggregates_active ON post_aggregates (hot_rank_active DESC);

CREATE INDEX idx_comment_aggregates_hot ON comment_aggregates (hot_rank DESC);

CREATE INDEX idx_category_aggregates_hot ON category_aggregates (hot_rank DESC);

