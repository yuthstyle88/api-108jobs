DROP VIEW category_view;

DROP VIEW post_view;

ALTER TABLE category
    DROP COLUMN self_promotion;

ALTER TABLE post
    DROP COLUMN self_promotion;

ALTER TABLE user_
    DROP COLUMN self_promotion;

-- the views
CREATE VIEW category_view AS
with all_category AS (
    SELECT
        *,
        (
            SELECT
                name
            FROM
                user_ u
            WHERE
                c.creator_id = u.id) AS creator_name,
        (
            SELECT
                name
            FROM
                category ct
            WHERE
                c.category_id = ct.id) AS category_name,
        (
            SELECT
                count(*)
            FROM
                category_follower cf
            WHERE
                cf.category_id = c.id) AS number_of_subscribers,
        (
            SELECT
                count(*)
            FROM
                post p
            WHERE
                p.category_id = c.id) AS number_of_posts,
        (
            SELECT
                count(*)
            FROM
                comment co,
                post p
            WHERE
                c.id = p.category_id
                AND p.id = co.post_id) AS number_of_comments,
        hot_rank ((
            SELECT
                count(*)
            FROM category_follower cf
            WHERE
                cf.category_id = c.id), c.published) AS hot_rank
FROM
    category c
)
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
    CROSS JOIN all_category ac
UNION ALL
SELECT
    ac.*,
    NULL AS user_id,
    NULL AS subscribed
FROM
    all_category ac;

-- Post view
CREATE VIEW post_view AS
with all_post AS (
    SELECT
        p.*,
        (
            SELECT
                name
            FROM
                user_
            WHERE
                p.creator_id = user_.id) AS creator_name,
        (
            SELECT
                name
            FROM
                category
            WHERE
                p.category_id = category.id) AS category_name,
        (
            SELECT
                removed
            FROM
                category c
            WHERE
                p.category_id = c.id) AS category_removed,
        (
            SELECT
                deleted
            FROM
                category c
            WHERE
                p.category_id = c.id) AS category_deleted,
        (
            SELECT
                count(*)
            FROM
                comment
            WHERE
                comment.post_id = p.id) AS number_of_comments,
        coalesce(sum(pl.score), 0) AS score,
        count(
            CASE WHEN pl.score = 1 THEN
                1
            ELSE
                NULL
            END) AS upvotes,
        count(
            CASE WHEN pl.score = -1 THEN
                1
            ELSE
                NULL
            END) AS downvotes,
        hot_rank (coalesce(sum(pl.score), 0), p.published) AS hot_rank
    FROM
        post p
        LEFT JOIN post_like pl ON p.id = pl.post_id
    GROUP BY
        p.id
)
SELECT
    ap.*,
    u.id AS user_id,
    coalesce(pl.score, 0) AS my_vote,
    (
        SELECT
            cf.id::bool
        FROM
            category_follower cf
        WHERE
            u.id = cf.user_id
            AND cf.category_id = ap.category_id) AS subscribed,
    (
        SELECT
            pr.id::bool
        FROM
            post_read pr
        WHERE
            u.id = pr.user_id
            AND pr.post_id = ap.id) AS read,
    (
        SELECT
            ps.id::bool
        FROM
            post_saved ps
        WHERE
            u.id = ps.user_id
            AND ps.post_id = ap.id) AS saved
FROM
    user_ u
    CROSS JOIN all_post ap
    LEFT JOIN post_like pl ON u.id = pl.user_id
        AND ap.id = pl.post_id
    UNION ALL
    SELECT
        ap.*,
        NULL AS user_id,
        NULL AS my_vote,
        NULL AS subscribed,
        NULL AS read,
        NULL AS saved
    FROM
        all_post ap;

