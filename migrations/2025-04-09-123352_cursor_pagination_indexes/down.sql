DROP INDEX idx_tagline_published_id;

DROP INDEX idx_comment_actions_like_score;

DROP INDEX idx_post_actions_like_score;

-- Fixing the category sorts for an id tie-breaker
DROP INDEX idx_category_lower_name;

DROP INDEX idx_category_hot;

DROP INDEX idx_category_published;

DROP INDEX idx_category_subscribers;

DROP INDEX idx_category_title;

DROP INDEX idx_category_users_active_month;

CREATE INDEX idx_category_lower_name ON category USING btree (lower((name)::text));

CREATE INDEX idx_category_hot ON category USING btree (hot_rank DESC);

CREATE INDEX idx_category_published ON category USING btree (published DESC);

CREATE INDEX idx_category_subscribers ON category USING btree (subscribers DESC);

CREATE INDEX idx_category_title ON category USING btree (title);

CREATE INDEX idx_category_users_active_month ON category USING btree (users_active_month DESC);

-- Drop the missing ones.
DROP INDEX idx_category_users_active_half_year;

DROP INDEX idx_category_users_active_week;

DROP INDEX idx_category_users_active_day;

DROP INDEX idx_category_subscribers_local;

DROP INDEX idx_category_comments;

DROP INDEX idx_category_posts;

-- Fix the post reverse_timestamp key sorts.
DROP INDEX idx_post_category_published;

DROP INDEX idx_post_featured_category_published;

CREATE INDEX idx_post_featured_category_published_asc ON post USING btree (category_id, featured_category DESC, reverse_timestamp_sort (published) DESC, id DESC);

CREATE INDEX idx_post_featured_local_published_asc ON post USING btree (featured_local DESC, reverse_timestamp_sort (published) DESC, id DESC);

CREATE INDEX idx_post_published_asc ON post USING btree (reverse_timestamp_sort (published) DESC);

