-- move comment_aggregates back into separate table
CREATE TABLE comment_aggregates (
    comment_id int PRIMARY KEY NOT NULL REFERENCES COMMENT ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE INITIALLY DEFERRED,
    score bigint NOT NULL DEFAULT 0,
    upvotes bigint NOT NULL DEFAULT 0,
    downvotes bigint NOT NULL DEFAULT 0,
    published timestamp with time zone NOT NULL DEFAULT now(),
    child_count integer NOT NULL DEFAULT 0,
    hot_rank double precision NOT NULL DEFAULT 0.0001,
    controversy_rank double precision NOT NULL DEFAULT 0,
    report_count smallint NOT NULL DEFAULT 0,
    unresolved_report_count smallint NOT NULL DEFAULT 0
);

INSERT INTO comment_aggregates
SELECT
    id AS comment_id,
    score,
    upvotes,
    downvotes,
    published,
    child_count,
    hot_rank,
    controversy_rank,
    report_count,
    unresolved_report_count
FROM
    comment;

ALTER TABLE comment
    DROP COLUMN score,
    DROP COLUMN upvotes,
    DROP COLUMN downvotes,
    DROP COLUMN child_count,
    DROP COLUMN hot_rank,
    DROP COLUMN controversy_rank,
    DROP COLUMN report_count,
    DROP COLUMN unresolved_report_count;

SET CONSTRAINTS comment_aggregates_comment_id_fkey IMMEDIATE;

SET CONSTRAINTS comment_aggregates_comment_id_fkey DEFERRED;

CREATE INDEX idx_comment_aggregates_controversy ON comment_aggregates USING btree (controversy_rank DESC);

CREATE INDEX idx_comment_aggregates_hot ON comment_aggregates USING btree (hot_rank DESC, score DESC);

CREATE INDEX idx_comment_aggregates_nonzero_hotrank ON comment_aggregates USING btree (published)
WHERE (hot_rank <> (0)::double precision);

CREATE INDEX idx_comment_aggregates_published ON comment_aggregates USING btree (published DESC);

CREATE INDEX idx_comment_aggregates_score ON comment_aggregates USING btree (score DESC);

-- move comment_aggregates back into separate table
CREATE TABLE post_aggregates (
    post_id int PRIMARY KEY NOT NULL REFERENCES post ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE INITIALLY DEFERRED,
    comments bigint NOT NULL DEFAULT 0,
    score bigint NOT NULL DEFAULT 0,
    upvotes bigint NOT NULL DEFAULT 0,
    downvotes bigint NOT NULL DEFAULT 0,
    published timestamp with time zone NOT NULL DEFAULT now(),
    newest_comment_time_necro timestamp with time zone NOT NULL DEFAULT now(),
    newest_comment_time timestamp with time zone NOT NULL DEFAULT now(),
    featured_category boolean NOT NULL DEFAULT FALSE,
    featured_local boolean NOT NULL DEFAULT FALSE,
    hot_rank double precision NOT NULL DEFAULT 0.0001,
    hot_rank_active double precision NOT NULL DEFAULT 0.0001,
    category_id integer NOT NULL REFERENCES category (id) ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE INITIALLY DEFERRED,
    creator_id integer NOT NULL REFERENCES person (id) ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE INITIALLY DEFERRED,
    controversy_rank double precision NOT NULL DEFAULT 0,
    instance_id integer NOT NULL REFERENCES instance (id) ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE INITIALLY DEFERRED,
    scaled_rank double precision NOT NULL DEFAULT 0.0001,
    report_count smallint NOT NULL DEFAULT 0,
    unresolved_report_count smallint NOT NULL DEFAULT 0
);

INSERT INTO post_aggregates
SELECT
    id AS post_id,
    comments,
    score,
    upvotes,
    downvotes,
    published,
    newest_comment_time_necro,
    newest_comment_time,
    featured_category,
    featured_local,
    hot_rank,
    hot_rank_active,
    category_id,
    creator_id,
    controversy_rank,
    instance_id,
    scaled_rank,
    report_count,
    unresolved_report_count
FROM
    post;

ALTER TABLE post
    DROP COLUMN comments,
    DROP COLUMN score,
    DROP COLUMN upvotes,
    DROP COLUMN downvotes,
    DROP COLUMN newest_comment_time_necro,
    DROP COLUMN newest_comment_time,
    DROP COLUMN hot_rank,
    DROP COLUMN hot_rank_active,
    DROP COLUMN controversy_rank,
    DROP COLUMN instance_id,
    DROP COLUMN scaled_rank,
    DROP COLUMN report_count,
    DROP COLUMN unresolved_report_count;

SET CONSTRAINTS post_aggregates_category_id_fkey, post_aggregates_creator_id_fkey, post_aggregates_instance_id_fkey, post_aggregates_post_id_fkey IMMEDIATE;

SET CONSTRAINTS post_aggregates_category_id_fkey, post_aggregates_creator_id_fkey, post_aggregates_instance_id_fkey, post_aggregates_post_id_fkey DEFERRED;

CREATE INDEX idx_post_aggregates_category_active ON post_aggregates USING btree (category_id, featured_local DESC, hot_rank_active DESC, published DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_category_controversy ON post_aggregates USING btree (category_id, featured_local DESC, controversy_rank DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_category_hot ON post_aggregates USING btree (category_id, featured_local DESC, hot_rank DESC, published DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_category_most_comments ON post_aggregates USING btree (category_id, featured_local DESC, comments DESC, published DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_category_newest_comment_time ON post_aggregates USING btree (category_id, featured_local DESC, newest_comment_time DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_category_newest_comment_time_necro ON post_aggregates USING btree (category_id, featured_local DESC, newest_comment_time_necro DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_category_published ON post_aggregates USING btree (category_id, featured_local DESC, published DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_category_published_asc ON post_aggregates USING btree (category_id, featured_local DESC, reverse_timestamp_sort (published) DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_category_scaled ON post_aggregates USING btree (category_id, featured_local DESC, scaled_rank DESC, published DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_category_score ON post_aggregates USING btree (category_id, featured_local DESC, score DESC, published DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_category_active ON post_aggregates USING btree (category_id, featured_category DESC, hot_rank_active DESC, published DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_category_controversy ON post_aggregates USING btree (category_id, featured_category DESC, controversy_rank DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_category_hot ON post_aggregates USING btree (category_id, featured_category DESC, hot_rank DESC, published DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_category_most_comments ON post_aggregates USING btree (category_id, featured_category DESC, comments DESC, published DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_category_newest_comment_time ON post_aggregates USING btree (category_id, featured_category DESC, newest_comment_time DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_category_newest_comment_time_necr ON post_aggregates USING btree (category_id, featured_category DESC, newest_comment_time_necro DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_category_published ON post_aggregates USING btree (category_id, featured_category DESC, published DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_category_published_asc ON post_aggregates USING btree (category_id, featured_category DESC, reverse_timestamp_sort (published) DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_category_scaled ON post_aggregates USING btree (category_id, featured_category DESC, scaled_rank DESC, published DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_category_score ON post_aggregates USING btree (category_id, featured_category DESC, score DESC, published DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_local_active ON post_aggregates USING btree (featured_local DESC, hot_rank_active DESC, published DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_local_controversy ON post_aggregates USING btree (featured_local DESC, controversy_rank DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_local_hot ON post_aggregates USING btree (featured_local DESC, hot_rank DESC, published DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_local_most_comments ON post_aggregates USING btree (featured_local DESC, comments DESC, published DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_local_newest_comment_time ON post_aggregates USING btree (featured_local DESC, newest_comment_time DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_local_newest_comment_time_necro ON post_aggregates USING btree (featured_local DESC, newest_comment_time_necro DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_local_published ON post_aggregates USING btree (featured_local DESC, published DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_local_published_asc ON post_aggregates USING btree (featured_local DESC, reverse_timestamp_sort (published) DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_local_scaled ON post_aggregates USING btree (featured_local DESC, scaled_rank DESC, published DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_featured_local_score ON post_aggregates USING btree (featured_local DESC, score DESC, published DESC, post_id DESC);

CREATE INDEX idx_post_aggregates_nonzero_hotrank ON post_aggregates USING btree (published DESC)
WHERE ((hot_rank <> (0)::double precision) OR (hot_rank_active <> (0)::double precision));

CREATE INDEX idx_post_aggregates_published ON post_aggregates USING btree (published DESC);

CREATE INDEX idx_post_aggregates_published_asc ON post_aggregates USING btree (reverse_timestamp_sort (published) DESC);

DROP INDEX idx_post_featured_category_published_asc;

DROP INDEX idx_post_featured_local_published;

DROP INDEX idx_post_featured_local_published_asc;

DROP INDEX idx_post_published;

DROP INDEX idx_post_published_asc;

DROP INDEX idx_search_combined_score;

-- move category_aggregates back into separate table
CREATE TABLE category_aggregates (
    category_id int PRIMARY KEY NOT NULL REFERENCES COMMunity ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE INITIALLY DEFERRED,
    subscribers bigint NOT NULL DEFAULT 0,
    posts bigint NOT NULL DEFAULT 0,
    comments bigint NOT NULL DEFAULT 0,
    published timestamp with time zone DEFAULT now() NOT NULL,
    users_active_day bigint NOT NULL DEFAULT 0,
    users_active_week bigint NOT NULL DEFAULT 0,
    users_active_month bigint NOT NULL DEFAULT 0,
    users_active_half_year bigint NOT NULL DEFAULT 0,
    hot_rank double precision NOT NULL DEFAULT 0.0001,
    subscribers_local bigint NOT NULL DEFAULT 0,
    report_count smallint NOT NULL DEFAULT 0,
    unresolved_report_count smallint NOT NULL DEFAULT 0,
    interactions_month bigint NOT NULL DEFAULT 0
);

INSERT INTO category_aggregates
SELECT
    id AS comment_id,
    subscribers,
    posts,
    comments,
    published,
    users_active_day,
    users_active_week,
    users_active_month,
    users_active_half_year,
    hot_rank,
    subscribers_local,
    report_count,
    unresolved_report_count,
    interactions_month
FROM
    category;

ALTER TABLE category
    DROP COLUMN subscribers,
    DROP COLUMN posts,
    DROP COLUMN comments,
    DROP COLUMN users_active_day,
    DROP COLUMN users_active_week,
    DROP COLUMN users_active_month,
    DROP COLUMN users_active_half_year,
    DROP COLUMN hot_rank,
    DROP COLUMN subscribers_local,
    DROP COLUMN report_count,
    DROP COLUMN unresolved_report_count,
    DROP COLUMN interactions_month,
    ALTER CONSTRAINT category_instance_id_fkey NOT DEFERRABLE INITIALLY IMMEDIATE;

SET CONSTRAINTS category_aggregates_category_id_fkey IMMEDIATE;

SET CONSTRAINTS category_aggregates_category_id_fkey DEFERRED;

CREATE INDEX idx_category_aggregates_hot ON public.category_aggregates USING btree (hot_rank DESC);

CREATE INDEX idx_category_aggregates_nonzero_hotrank ON public.category_aggregates USING btree (published)
WHERE (hot_rank <> (0)::double precision);

CREATE INDEX idx_category_aggregates_published ON public.category_aggregates USING btree (published DESC);

CREATE INDEX idx_category_aggregates_subscribers ON public.category_aggregates USING btree (subscribers DESC);

CREATE INDEX idx_category_aggregates_users_active_month ON public.category_aggregates USING btree (users_active_month DESC);

-- move person_aggregates back into separate table
CREATE TABLE person_aggregates (
    person_id int PRIMARY KEY NOT NULL REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE INITIALLY DEFERRED,
    post_count bigint NOT NULL DEFAULT 0,
    post_score bigint NOT NULL DEFAULT 0,
    comment_count bigint NOT NULL DEFAULT 0,
    comment_score bigint NOT NULL DEFAULT 0,
    published timestamp with time zone DEFAULT now() NOT NULL
);

INSERT INTO person_aggregates
SELECT
    id AS person_id,
    post_count,
    post_score,
    comment_count,
    comment_score,
    published
FROM
    person;

ALTER TABLE person
    DROP COLUMN post_count,
    DROP COLUMN post_score,
    DROP COLUMN comment_count,
    DROP COLUMN comment_score;

SET CONSTRAINTS person_aggregates_person_id_fkey IMMEDIATE;

SET CONSTRAINTS person_aggregates_person_id_fkey DEFERRED;

CREATE INDEX idx_person_aggregates_comment_score ON public.person_aggregates USING btree (comment_score DESC);

CREATE INDEX idx_person_aggregates_person ON public.person_aggregates USING btree (person_id);

-- move site_aggregates back into separate table
CREATE TABLE site_aggregates (
    site_id int PRIMARY KEY NOT NULL REFERENCES site ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE INITIALLY DEFERRED,
    users bigint NOT NULL DEFAULT 1,
    posts bigint NOT NULL DEFAULT 0,
    comments bigint NOT NULL DEFAULT 0,
    communities bigint NOT NULL DEFAULT 0,
    users_active_day bigint NOT NULL DEFAULT 0,
    users_active_week bigint NOT NULL DEFAULT 0,
    users_active_month bigint NOT NULL DEFAULT 0,
    users_active_half_year bigint NOT NULL DEFAULT 0
);

INSERT INTO site_aggregates
SELECT
    id AS site_id,
    users,
    posts,
    comments,
    communities,
    users_active_day,
    users_active_week,
    users_active_month,
    users_active_half_year
FROM
    local_site;

ALTER TABLE local_site
    DROP COLUMN users,
    DROP COLUMN posts,
    DROP COLUMN comments,
    DROP COLUMN communities,
    DROP COLUMN users_active_day,
    DROP COLUMN users_active_week,
    DROP COLUMN users_active_month,
    DROP COLUMN users_active_half_year;

-- move local_user_vote_display_mode back into separate table
CREATE TABLE local_user_vote_display_mode (
    local_user_id int PRIMARY KEY NOT NULL REFERENCES local_user ON UPDATE CASCADE ON DELETE CASCADE,
    score boolean NOT NULL DEFAULT FALSE,
    upvotes boolean NOT NULL DEFAULT TRUE,
    downvotes boolean NOT NULL DEFAULT TRUE,
    upvote_percentage boolean NOT NULL DEFAULT FALSE
);

INSERT INTO local_user_vote_display_mode
SELECT
    id AS local_user_id,
    show_score AS score,
    show_upvotes AS upvotes,
    show_downvotes AS downvotes,
    show_upvote_percentage AS upvote_percentage
FROM
    local_user;

ALTER TABLE local_user
    DROP COLUMN show_score,
    DROP COLUMN show_upvotes,
    DROP COLUMN show_downvotes,
    DROP COLUMN show_upvote_percentage;

CREATE INDEX idx_search_combined_score ON public.search_combined USING btree (score DESC, id DESC);

SET CONSTRAINTS site_aggregates_site_id_fkey IMMEDIATE;

SET CONSTRAINTS site_aggregates_site_id_fkey DEFERRED;

CREATE UNIQUE INDEX idx_site_aggregates_1_row_only ON public.site_aggregates USING btree ((TRUE));

