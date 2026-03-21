-- Drop first
DROP VIEW category_view;

DROP VIEW category_aggregates_view;

DROP VIEW category_fast_view;

DROP TABLE category_aggregates_fast;

CREATE VIEW category_aggregates_view AS
SELECT
    c.id,
    c.name,
    c.title,
    c.icon,
    c.banner,
    c.description,
    c.category_id,
    c.creator_id,
    c.removed,
    c.published,
    c.updated,
    c.deleted,
    c.self_promotion,
    c.actor_id,
    c.local,
    c.last_refreshed_at,
    u.actor_id AS creator_actor_id,
    u.local AS creator_local,
    u.name AS creator_name,
    u.preferred_username AS creator_preferred_username,
    u.avatar AS creator_avatar,
    cat.name AS category_name,
    coalesce(cf.subs, 0) AS number_of_subscribers,
    coalesce(cd.posts, 0) AS number_of_posts,
    coalesce(cd.comments, 0) AS number_of_comments,
    hot_rank (cf.subs, c.published) AS hot_rank
FROM
    category c
    LEFT JOIN user_ u ON c.creator_id = u.id
    LEFT JOIN category cat ON c.category_id = cat.id
    LEFT JOIN (
        SELECT
            p.category_id,
            count(DISTINCT p.id) AS posts,
            count(DISTINCT ct.id) AS comments
        FROM
            post p
            JOIN comment ct ON p.id = ct.post_id
        GROUP BY
            p.category_id) cd ON cd.category_id = c.id
    LEFT JOIN (
        SELECT
            category_id,
            count(*) AS subs
        FROM
            category_follower
        GROUP BY
            category_id) cf ON cf.category_id = c.id;

CREATE VIEW category_view AS
SELECT
    cv.*,
    us.user AS user_id,
    us.is_subbed::bool AS subscribed
FROM
    category_aggregates_view cv
    CROSS JOIN LATERAL (
        SELECT
            u.id AS user,
            coalesce(cf.category_id, 0) AS is_subbed
        FROM
            user_ u
            LEFT JOIN category_follower cf ON u.id = cf.user_id
                AND cf.category_id = cv.id) AS us
UNION ALL
SELECT
    cv.*,
    NULL AS user_id,
    NULL AS subscribed
FROM
    category_aggregates_view cv;

-- The category fast table
CREATE TABLE category_aggregates_fast AS
SELECT
    *
FROM
    category_aggregates_view;

ALTER TABLE category_aggregates_fast
    ADD PRIMARY KEY (id);

CREATE VIEW category_fast_view AS
SELECT
    ac.*,
    u.id AS user_id,
    (
        SELECT
            cf.id::boolean
        FROM
            category_follower cf
        WHERE
            u.id = cf.user_id
            AND ac.id = cf.category_id) AS subscribed
FROM
    user_ u
    CROSS JOIN (
        SELECT
            ca.*
        FROM
            category_aggregates_fast ca) ac
UNION ALL
SELECT
    caf.*,
    NULL AS user_id,
    NULL AS subscribed
FROM
    category_aggregates_fast caf;

