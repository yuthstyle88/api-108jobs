-- recreate columns in the original order
ALTER TABLE category
    ADD COLUMN hidden bool DEFAULT FALSE NOT NULL,
    ADD COLUMN posting_restricted_to_mods_new bool NOT NULL DEFAULT FALSE,
    ADD COLUMN instance_id_new int,
    ADD COLUMN moderators_url_new varchar(255),
    ADD COLUMN featured_url_new varchar(255),
    ADD COLUMN visibility_new category_visibility NOT NULL DEFAULT 'Public',
    ADD COLUMN description_new varchar(150),
    ADD COLUMN random_number_new smallint NOT NULL DEFAULT random_smallint (),
    ADD COLUMN subscribers_new bigint NOT NULL DEFAULT 0,
    ADD COLUMN posts_new bigint NOT NULL DEFAULT 0,
    ADD COLUMN comments_new bigint NOT NULL DEFAULT 0,
    ADD COLUMN users_active_day_new bigint NOT NULL DEFAULT 0,
    ADD COLUMN users_active_week_new bigint NOT NULL DEFAULT 0,
    ADD COLUMN users_active_month_new bigint NOT NULL DEFAULT 0,
    ADD COLUMN users_active_half_year_new bigint NOT NULL DEFAULT 0,
    ADD COLUMN hot_rank_new double precision NOT NULL DEFAULT 0.0001,
    ADD COLUMN subscribers_local_new bigint NOT NULL DEFAULT 0,
    ADD COLUMN report_count_new smallint NOT NULL DEFAULT 0,
    ADD COLUMN unresolved_report_count_new smallint NOT NULL DEFAULT 0,
    ADD COLUMN interactions_month_new bigint NOT NULL DEFAULT 0;

UPDATE
    category
SET
    (posting_restricted_to_mods_new,
        instance_id_new,
        moderators_url_new,
        featured_url_new,
        visibility_new,
        description_new,
        random_number_new,
        subscribers_new,
        posts_new,
        comments_new,
        users_active_day_new,
        users_active_week_new,
        users_active_month_new,
        users_active_half_year_new,
        hot_rank_new,
        subscribers_local_new,
        report_count_new,
        unresolved_report_count_new,
        interactions_month_new) = (posting_restricted_to_mods,
        instance_id,
        moderators_url,
        featured_url,
        visibility,
        description,
        random_number,
        subscribers,
        posts,
        comments,
        users_active_day,
        users_active_week,
        users_active_month,
        users_active_half_year,
        hot_rank,
        subscribers_local,
        report_count,
        unresolved_report_count,
        interactions_month);

SET CONSTRAINTS category_instance_id_fkey IMMEDIATE;

SET CONSTRAINTS category_instance_id_fkey DEFERRED;

ALTER TABLE category
    ALTER COLUMN instance_id_new SET NOT NULL,
    DROP COLUMN posting_restricted_to_mods,
    DROP COLUMN instance_id,
    DROP COLUMN moderators_url,
    DROP COLUMN featured_url,
    DROP COLUMN visibility,
    DROP COLUMN description,
    DROP COLUMN random_number,
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
    DROP COLUMN interactions_month;

ALTER TABLE category RENAME COLUMN posting_restricted_to_mods_new TO posting_restricted_to_mods;

ALTER TABLE category RENAME COLUMN instance_id_new TO instance_id;

ALTER TABLE category RENAME COLUMN moderators_url_new TO moderators_url;

ALTER TABLE category RENAME COLUMN featured_url_new TO featured_url;

ALTER TABLE category RENAME COLUMN visibility_new TO visibility;

ALTER TABLE category RENAME COLUMN description_new TO description;

ALTER TABLE category RENAME COLUMN random_number_new TO random_number;

ALTER TABLE category RENAME COLUMN subscribers_new TO subscribers;

ALTER TABLE category RENAME COLUMN posts_new TO posts;

ALTER TABLE category RENAME COLUMN comments_new TO comments;

ALTER TABLE category RENAME COLUMN users_active_day_new TO users_active_day;

ALTER TABLE category RENAME COLUMN users_active_week_new TO users_active_week;

ALTER TABLE category RENAME COLUMN users_active_month_new TO users_active_month;

ALTER TABLE category RENAME COLUMN users_active_half_year_new TO users_active_half_year;

ALTER TABLE category RENAME COLUMN hot_rank_new TO hot_rank;

ALTER TABLE category RENAME COLUMN subscribers_local_new TO subscribers_local;

ALTER TABLE category RENAME COLUMN report_count_new TO report_count;

ALTER TABLE category RENAME COLUMN unresolved_report_count_new TO unresolved_report_count;

ALTER TABLE category RENAME COLUMN interactions_month_new TO interactions_month;

ALTER TABLE category
    ADD CONSTRAINT category_featured_url_key UNIQUE (featured_url),
    ADD CONSTRAINT category_moderators_url_key UNIQUE (moderators_url),
    ADD CONSTRAINT category_instance_id_fkey FOREIGN KEY (instance_id) REFERENCES instance (id) ON UPDATE CASCADE ON DELETE CASCADE DEFERRABLE INITIALLY DEFERRED;

-- same changes as up.sql, but the other way round
UPDATE
    category
SET
    (hidden,
        visibility) = (TRUE,
        'Public')
WHERE
    visibility = 'Unlisted';

ALTER TYPE category_visibility RENAME VALUE 'LocalOnlyPrivate' TO 'LocalOnly';

ALTER TYPE category_visibility RENAME TO category_visibility__;

CREATE TYPE category_visibility AS enum (
    'Public',
    'LocalOnly',
    'Private'
);

ALTER TABLE category
    ALTER COLUMN visibility DROP DEFAULT;

ALTER TABLE category
    ALTER COLUMN visibility TYPE category_visibility
    USING visibility::text::category_visibility;

ALTER TABLE category
    ALTER COLUMN visibility SET DEFAULT 'Public';

CREATE INDEX idx_category_random_number ON category (random_number) INCLUDE (local, self_promotion)
WHERE
    NOT (deleted OR removed OR visibility = 'Private');

CREATE INDEX idx_category_nonzero_hotrank ON category USING btree (published)
WHERE (hot_rank <> (0)::double precision);

CREATE INDEX idx_category_subscribers ON category USING btree (subscribers DESC);

CREATE INDEX idx_category_users_active_month ON category USING btree (users_active_month DESC);

CREATE INDEX idx_category_hot ON public.category USING btree (hot_rank DESC);

REINDEX TABLE category;

-- revert modlog table changes
CREATE TABLE mod_hide_category (
    id serial PRIMARY KEY,
    category_id int REFERENCES category ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    mod_person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    published timestamptz NOT NULL DEFAULT now(),
    reason text,
    hidden boolean DEFAULT FALSE NOT NULL
);

ALTER TABLE modlog_combined
    DROP COLUMN mod_change_category_visibility_id,
    ADD COLUMN mod_hide_category_id int REFERENCES mod_hide_category ON UPDATE CASCADE ON DELETE CASCADE,
    ADD COLUMN mod_lock_post_id_new int,
    ADD COLUMN mod_remove_comment_id_new int,
    ADD COLUMN mod_remove_category_id_new int,
    ADD COLUMN mod_remove_post_id_new int,
    ADD COLUMN mod_transfer_category_id_new int;

UPDATE
    modlog_combined
SET
    (mod_lock_post_id_new,
        mod_remove_comment_id_new,
        mod_remove_category_id_new,
        mod_remove_post_id_new,
        mod_transfer_category_id_new) = (mod_lock_post_id,
        mod_remove_comment_id,
        mod_remove_category_id,
        mod_remove_post_id,
        mod_transfer_category_id);

ALTER TABLE modlog_combined
    DROP COLUMN mod_lock_post_id,
    DROP COLUMN mod_remove_comment_id,
    DROP COLUMN mod_remove_category_id,
    DROP COLUMN mod_remove_post_id,
    DROP COLUMN mod_transfer_category_id;

ALTER TABLE modlog_combined RENAME COLUMN mod_lock_post_id_new TO mod_lock_post_id;

ALTER TABLE modlog_combined RENAME COLUMN mod_remove_comment_id_new TO mod_remove_comment_id;

ALTER TABLE modlog_combined RENAME COLUMN mod_remove_category_id_new TO mod_remove_category_id;

ALTER TABLE modlog_combined RENAME COLUMN mod_remove_post_id_new TO mod_remove_post_id;

ALTER TABLE modlog_combined RENAME COLUMN mod_transfer_category_id_new TO mod_transfer_category_id;

ALTER TABLE modlog_combined
    ADD CONSTRAINT modlog_combined_mod_hide_category_id_key UNIQUE (mod_hide_category_id),
    ADD CONSTRAINT modlog_combined_mod_lock_post_id_key UNIQUE (mod_lock_post_id),
    ADD CONSTRAINT modlog_combined_mod_remove_comment_id_key UNIQUE (mod_remove_comment_id),
    ADD CONSTRAINT modlog_combined_mod_remove_category_id_key UNIQUE (mod_remove_category_id),
    ADD CONSTRAINT modlog_combined_mod_remove_post_id_key UNIQUE (mod_remove_post_id),
    ADD CONSTRAINT modlog_combined_mod_transfer_category_id_key UNIQUE (mod_transfer_category_id),
    ADD CONSTRAINT modlog_combined_mod_lock_post_id_fkey FOREIGN KEY (mod_lock_post_id) REFERENCES mod_lock_post (id) ON UPDATE CASCADE ON DELETE CASCADE,
    ADD CONSTRAINT modlog_combined_mod_remove_comment_id_fkey FOREIGN KEY (mod_remove_comment_id) REFERENCES mod_remove_comment (id) ON UPDATE CASCADE ON DELETE CASCADE,
    ADD CONSTRAINT modlog_combined_mod_remove_category_id_fkey FOREIGN KEY (mod_remove_category_id) REFERENCES mod_remove_category (id) ON UPDATE CASCADE ON DELETE CASCADE,
    ADD CONSTRAINT modlog_combined_mod_remove_post_id_fkey FOREIGN KEY (mod_remove_post_id) REFERENCES mod_remove_post (id) ON UPDATE CASCADE ON DELETE CASCADE,
    ADD CONSTRAINT modlog_combined_mod_transfer_category_id_fkey FOREIGN KEY (mod_transfer_category_id) REFERENCES mod_transfer_category (id) ON UPDATE CASCADE ON DELETE CASCADE,
    ADD CONSTRAINT modlog_combined_check CHECK ((num_nonnulls (admin_allow_instance_id, admin_block_instance_id, admin_purge_comment_id, admin_purge_category_id, admin_purge_person_id, admin_purge_post_id, mod_add_id, mod_add_category_id, mod_ban_id, mod_ban_from_category_id, mod_feature_post_id, mod_hide_category_id, mod_lock_post_id, mod_remove_comment_id, mod_remove_category_id, mod_remove_post_id, mod_transfer_category_id) = 1));

DROP TABLE mod_change_category_visibility;

DROP TYPE category_visibility__;

