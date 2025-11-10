-- the views
DROP VIEW user_mention_view;

DROP VIEW reply_view;

DROP VIEW comment_view;

DROP VIEW user_view;

-- user
CREATE VIEW user_view AS
SELECT
    id,
    name,
    fedi_name,
    admin,
    banned,
    published,
    (
        SELECT
            count(*)
        FROM
            post p
        WHERE
            p.creator_id = u.id) AS number_of_posts,
    (
        SELECT
            coalesce(sum(score), 0)
        FROM
            post p,
            post_like pl
        WHERE
            u.id = p.creator_id
            AND p.id = pl.post_id) AS post_score,
    (
        SELECT
            count(*)
        FROM
            comment c
        WHERE
            c.creator_id = u.id) AS number_of_comments,
    (
        SELECT
            coalesce(sum(score), 0)
        FROM
            comment c,
            comment_like cl
        WHERE
            u.id = c.creator_id
            AND c.id = cl.comment_id) AS comment_score
FROM
    user_ u;

-- post
-- Recreate the view
DROP VIEW post_view;

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

-- category
DROP VIEW category_view;

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

-- Reply and comment view
CREATE VIEW comment_view AS
with all_comment AS (
    SELECT
        c.*,
        (
            SELECT
                category_id
            FROM
                post p
            WHERE
                p.id = c.post_id),
            (
                SELECT
                    u.banned
                FROM
                    user_ u
                WHERE
                    c.creator_id = u.id) AS banned,
                (
                    SELECT
                        cb.id::bool
                    FROM
                        category_user_ban cb,
                        post p
                    WHERE
                        c.creator_id = cb.user_id
                        AND p.id = c.post_id
                        AND p.category_id = cb.category_id) AS banned_from_category,
                    (
                        SELECT
                            name
                        FROM
                            user_
                        WHERE
                            c.creator_id = user_.id) AS creator_name,
                        coalesce(sum(cl.score), 0) AS score,
                    count(
                        CASE WHEN cl.score = 1 THEN
                            1
                        ELSE
                            NULL
                        END) AS upvotes,
                    count(
                        CASE WHEN cl.score = -1 THEN
                            1
                        ELSE
                            NULL
                        END) AS downvotes
                FROM
                    comment c
                LEFT JOIN comment_like cl ON c.id = cl.comment_id
            GROUP BY
                c.id
)
    SELECT
        ac.*,
        u.id AS user_id,
        coalesce(cl.score, 0) AS my_vote,
    (
        SELECT
            cs.id::bool
        FROM
            comment_saved cs
        WHERE
            u.id = cs.user_id
            AND cs.comment_id = ac.id) AS saved
FROM
    user_ u
    CROSS JOIN all_comment ac
    LEFT JOIN comment_like cl ON u.id = cl.user_id
        AND ac.id = cl.comment_id
    UNION ALL
    SELECT
        ac.*,
        NULL AS user_id,
        NULL AS my_vote,
        NULL AS saved
    FROM
        all_comment ac;

CREATE VIEW reply_view AS
with closereply AS (
    SELECT
        c2.id,
        c2.creator_id AS sender_id,
        c.creator_id AS recipient_id
    FROM
        comment c
        INNER JOIN comment c2 ON c.id = c2.parent_id
    WHERE
        c2.creator_id != c.creator_id
        -- Do union where post is null
    UNION
    SELECT
        c.id,
        c.creator_id AS sender_id,
        p.creator_id AS recipient_id
    FROM
        comment c,
        post p
    WHERE
        c.post_id = p.id
        AND c.parent_id IS NULL
        AND c.creator_id != p.creator_id
)
SELECT
    cv.*,
    closereply.recipient_id
FROM
    comment_view cv,
    closereply
WHERE
    closereply.id = cv.id;

-- user mention
CREATE VIEW user_mention_view AS
SELECT
    c.id,
    um.id AS user_mention_id,
    c.creator_id,
    c.post_id,
    c.parent_id,
    c.content,
    c.removed,
    um.read,
    c.published,
    c.updated,
    c.deleted,
    c.category_id,
    c.banned,
    c.banned_from_category,
    c.creator_name,
    c.score,
    c.upvotes,
    c.downvotes,
    c.user_id,
    c.my_vote,
    c.saved,
    um.recipient_id
FROM
    user_mention um,
    comment_view c
WHERE
    um.comment_id = c.id;

-- category tables
DROP VIEW category_moderator_view;

DROP VIEW category_follower_view;

DROP VIEW category_user_ban_view;

DROP VIEW site_view;

CREATE VIEW category_moderator_view AS
SELECT
    *,
    (
        SELECT
            name
        FROM
            user_ u
        WHERE
            cm.user_id = u.id) AS user_name,
    (
        SELECT
            name
        FROM
            category c
        WHERE
            cm.category_id = c.id) AS category_name
FROM
    category_moderator cm;

CREATE VIEW category_follower_view AS
SELECT
    *,
    (
        SELECT
            name
        FROM
            user_ u
        WHERE
            cf.user_id = u.id) AS user_name,
    (
        SELECT
            name
        FROM
            category c
        WHERE
            cf.category_id = c.id) AS category_name
FROM
    category_follower cf;

CREATE VIEW category_user_ban_view AS
SELECT
    *,
    (
        SELECT
            name
        FROM
            user_ u
        WHERE
            cm.user_id = u.id) AS user_name,
    (
        SELECT
            name
        FROM
            category c
        WHERE
            cm.category_id = c.id) AS category_name
FROM
    category_user_ban cm;

CREATE VIEW site_view AS
SELECT
    *,
    (
        SELECT
            name
        FROM
            user_ u
        WHERE
            s.creator_id = u.id) AS creator_name,
    (
        SELECT
            count(*)
        FROM
            user_) AS number_of_users,
    (
        SELECT
            count(*)
        FROM
            post) AS number_of_posts,
    (
        SELECT
            count(*)
        FROM
            comment) AS number_of_comments,
    (
        SELECT
            count(*)
        FROM
            category) AS number_of_communities
FROM
    site s;

ALTER TABLE user_ RENAME COLUMN avatar TO icon;

ALTER TABLE user_
    ALTER COLUMN icon TYPE bytea
    USING icon::bytea;

