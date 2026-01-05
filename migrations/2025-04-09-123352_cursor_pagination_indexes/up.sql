-- Taglines
CREATE INDEX idx_tagline_published_id ON tagline (published DESC, id DESC);

-- Some for the vote views
CREATE INDEX idx_comment_actions_like_score ON comment_actions (comment_id, like_score, person_id)
WHERE
    like_score IS NOT NULL;

CREATE INDEX idx_post_actions_like_score ON post_actions (post_id, like_score, person_id)
WHERE
    like_score IS NOT NULL;

-- Fixing the category sorts for an id tie-breaker
DROP INDEX idx_category_lower_name;

DROP INDEX idx_category_hot;

DROP INDEX idx_category_published;

DROP INDEX idx_category_subscribers;

DROP INDEX idx_category_title;

DROP INDEX idx_category_users_active_month;

CREATE INDEX idx_category_lower_name ON category USING btree (lower((name)::text) DESC, id DESC);

CREATE INDEX idx_category_hot ON category USING btree (hot_rank DESC, id DESC);

CREATE INDEX idx_category_published ON category USING btree (published DESC, id DESC);

CREATE INDEX idx_category_subscribers ON category USING btree (subscribers DESC, id DESC);

CREATE INDEX idx_category_title ON category USING btree (title DESC, id DESC);

CREATE INDEX idx_category_users_active_month ON category USING btree (users_active_month DESC, id DESC);

-- Create a few missing ones
CREATE INDEX idx_category_users_active_half_year ON category USING btree (users_active_half_year DESC, id DESC);

CREATE INDEX idx_category_users_active_week ON category USING btree (users_active_week DESC, id DESC);

CREATE INDEX idx_category_users_active_day ON category USING btree (users_active_day DESC, id DESC);

CREATE INDEX idx_category_subscribers_local ON category USING btree (subscribers_local DESC, id DESC);

CREATE INDEX idx_category_comments ON category USING btree (comments DESC, id DESC);

CREATE INDEX idx_category_posts ON category USING btree (posts DESC, id DESC);

-- Fix the post reverse_timestamp key sorts.
DROP INDEX idx_post_featured_category_published_asc;

DROP INDEX idx_post_featured_local_published_asc;

DROP INDEX idx_post_published_asc;

CREATE INDEX idx_post_featured_category_published ON post USING btree (category_id, featured_category DESC, published DESC, id DESC);

CREATE INDEX idx_post_category_published ON post USING btree (category_id, published DESC, id DESC);

