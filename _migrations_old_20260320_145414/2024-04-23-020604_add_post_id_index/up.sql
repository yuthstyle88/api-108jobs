-- Add , post_id DESC to all these
DROP INDEX idx_post_aggregates_category_active;

DROP INDEX idx_post_aggregates_category_controversy;

DROP INDEX idx_post_aggregates_category_hot;

DROP INDEX idx_post_aggregates_category_most_comments;

DROP INDEX idx_post_aggregates_category_newest_comment_time;

DROP INDEX idx_post_aggregates_category_newest_comment_time_necro;

DROP INDEX idx_post_aggregates_category_published;

DROP INDEX idx_post_aggregates_category_published_asc;

DROP INDEX idx_post_aggregates_category_scaled;

DROP INDEX idx_post_aggregates_category_score;

DROP INDEX idx_post_aggregates_featured_category_active;

DROP INDEX idx_post_aggregates_featured_category_controversy;

DROP INDEX idx_post_aggregates_featured_category_hot;

DROP INDEX idx_post_aggregates_featured_category_most_comments;

DROP INDEX idx_post_aggregates_featured_category_newest_comment_time;


DROP INDEX idx_post_aggregates_featured_category_published;

DROP INDEX idx_post_aggregates_featured_category_published_asc;

DROP INDEX idx_post_aggregates_featured_category_scaled;

DROP INDEX idx_post_aggregates_featured_category_score;

DROP INDEX idx_post_aggregates_featured_local_active;

DROP INDEX idx_post_aggregates_featured_local_controversy;

DROP INDEX idx_post_aggregates_featured_local_hot;

DROP INDEX idx_post_aggregates_featured_local_most_comments;

DROP INDEX idx_post_aggregates_featured_local_newest_comment_time;

DROP INDEX idx_post_aggregates_featured_local_newest_comment_time_necro;

DROP INDEX idx_post_aggregates_featured_local_published;

DROP INDEX idx_post_aggregates_featured_local_published_asc;

DROP INDEX idx_post_aggregates_featured_local_scaled;

DROP INDEX idx_post_aggregates_featured_local_score;

CREATE INDEX idx_post_aggregates_category_active ON public.post_aggregates USING btree (category_id, featured_local DESC, hot_rank_active DESC, published DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_category_controversy ON public.post_aggregates USING btree (category_id, featured_local DESC, controversy_rank DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_category_hot ON public.post_aggregates USING btree (category_id, featured_local DESC, hot_rank DESC, published DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_category_most_comments ON public.post_aggregates USING btree (category_id, featured_local DESC, comments DESC, published DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_category_newest_comment_time ON public.post_aggregates USING btree (category_id, featured_local DESC, newest_comment_time DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_category_newest_comment_time_necro ON public.post_aggregates USING btree (category_id, featured_local DESC, newest_comment_time_necro DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_category_published ON public.post_aggregates USING btree (category_id, featured_local DESC, published DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_category_published_asc ON public.post_aggregates USING btree (category_id, featured_local DESC, public.reverse_timestamp_sort (published) DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_category_scaled ON public.post_aggregates USING btree (category_id, featured_local DESC, scaled_rank DESC, published DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_category_score ON public.post_aggregates USING btree (category_id, featured_local DESC, score DESC, published DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_category_active ON public.post_aggregates USING btree (category_id, featured_category DESC, hot_rank_active DESC, published DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_category_controversy ON public.post_aggregates USING btree (category_id, featured_category DESC, controversy_rank DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_category_hot ON public.post_aggregates USING btree (category_id, featured_category DESC, hot_rank DESC, published DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_category_most_comments ON public.post_aggregates USING btree (category_id, featured_category DESC, comments DESC, published DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_category_newest_comment_time ON public.post_aggregates USING btree (category_id, featured_category DESC, newest_comment_time DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_category_newest_comment_time_necr ON public.post_aggregates USING btree (category_id, featured_category DESC, newest_comment_time_necro DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_category_published ON public.post_aggregates USING btree (category_id, featured_category DESC, published DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_category_published_asc ON public.post_aggregates USING btree (category_id, featured_category DESC, public.reverse_timestamp_sort (published) DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_category_scaled ON public.post_aggregates USING btree (category_id, featured_category DESC, scaled_rank DESC, published DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_category_score ON public.post_aggregates USING btree (category_id, featured_category DESC, score DESC, published DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_local_active ON public.post_aggregates USING btree (featured_local DESC, hot_rank_active DESC, published DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_local_controversy ON public.post_aggregates USING btree (featured_local DESC, controversy_rank DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_local_hot ON public.post_aggregates USING btree (featured_local DESC, hot_rank DESC, published DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_local_most_comments ON public.post_aggregates USING btree (featured_local DESC, comments DESC, published DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_local_newest_comment_time ON public.post_aggregates USING btree (featured_local DESC, newest_comment_time DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_local_newest_comment_time_necro ON public.post_aggregates USING btree (featured_local DESC, newest_comment_time_necro DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_local_published ON public.post_aggregates USING btree (featured_local DESC, published DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_local_published_asc ON public.post_aggregates USING btree (featured_local DESC, public.reverse_timestamp_sort (published) DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_local_scaled ON public.post_aggregates USING btree (featured_local DESC, scaled_rank DESC, published DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_local_score ON public.post_aggregates USING btree (featured_local DESC, score DESC, published DESC, post_id DESC);

