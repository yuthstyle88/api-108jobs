DROP VIEW post_view;

DROP VIEW mod_sticky_post_view;

ALTER TABLE post
    DROP COLUMN stickied;

DROP TABLE mod_sticky_post;

CREATE VIEW post_view AS
with all_post AS (
    SELECT
        p.*,
        (
            SELECT
                u.banned
            FROM
                user_ u
            WHERE
                p.creator_id = u.id) AS banned,
        (
            SELECT
                cb.id::bool
            FROM
                category_user_ban cb
            WHERE
                p.creator_id = cb.user_id
                AND p.category_id = cb.category_id) AS banned_from_category,
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
                self_promotion
            FROM
                category c
            WHERE
                p.category_id = c.id) AS category_self_promotion,
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

