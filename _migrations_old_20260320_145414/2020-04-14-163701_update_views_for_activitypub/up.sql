-- user_view
DROP VIEW user_view CASCADE;

CREATE VIEW user_view AS
SELECT
    u.id,
    u.actor_id,
    u.name,
    u.avatar,
    u.email,
    u.matrix_user_id,
    u.bio,
    u.local,
    u.admin,
    u.banned,
    u.show_avatars,
    u.send_notifications_to_email,
    u.published,
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

CREATE MATERIALIZED VIEW user_mview AS
SELECT
    *
FROM
    user_view;

CREATE UNIQUE INDEX idx_user_mview_id ON user_mview (id);

-- category_view
DROP VIEW category_aggregates_view CASCADE;

CREATE VIEW category_aggregates_view AS
-- Now that there's public and private keys, you have to be explicit here
SELECT
    c.id,
    c.name,
    c.title,
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
    (
        SELECT
            actor_id
        FROM
            user_ u
        WHERE
            c.creator_id = u.id) AS creator_actor_id,
    (
        SELECT
            local
        FROM
            user_ u
        WHERE
            c.creator_id = u.id) AS creator_local,
    (
        SELECT
            name
        FROM
            user_ u
        WHERE
            c.creator_id = u.id) AS creator_name,
    (
        SELECT
            avatar
        FROM
            user_ u
        WHERE
            c.creator_id = u.id) AS creator_avatar,
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
    category c;

CREATE MATERIALIZED VIEW category_aggregates_mview AS
SELECT
    *
FROM
    category_aggregates_view;

CREATE UNIQUE INDEX idx_category_aggregates_mview_id ON category_aggregates_mview (id);

CREATE VIEW category_view AS
with all_category AS (
    SELECT
        ca.*
    FROM
        category_aggregates_view ca
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

CREATE VIEW category_mview AS
with all_category AS (
    SELECT
        ca.*
    FROM
        category_aggregates_mview ca
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

-- category views
DROP VIEW category_moderator_view;

DROP VIEW category_follower_view;

DROP VIEW category_user_ban_view;

CREATE VIEW category_moderator_view AS
SELECT
    *,
    (
        SELECT
            actor_id
        FROM
            user_ u
        WHERE
            cm.user_id = u.id) AS user_actor_id,
    (
        SELECT
            local
        FROM
            user_ u
        WHERE
            cm.user_id = u.id) AS user_local,
    (
        SELECT
            name
        FROM
            user_ u
        WHERE
            cm.user_id = u.id) AS user_name,
    (
        SELECT
            avatar
        FROM
            user_ u
        WHERE
            cm.user_id = u.id), (
        SELECT
            actor_id
        FROM
            category c
        WHERE
            cm.category_id = c.id) AS category_actor_id,
    (
        SELECT
            local
        FROM
            category c
        WHERE
            cm.category_id = c.id) AS category_local,
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
            actor_id
        FROM
            user_ u
        WHERE
            cf.user_id = u.id) AS user_actor_id,
    (
        SELECT
            local
        FROM
            user_ u
        WHERE
            cf.user_id = u.id) AS user_local,
    (
        SELECT
            name
        FROM
            user_ u
        WHERE
            cf.user_id = u.id) AS user_name,
    (
        SELECT
            avatar
        FROM
            user_ u
        WHERE
            cf.user_id = u.id), (
        SELECT
            actor_id
        FROM
            category c
        WHERE
            cf.category_id = c.id) AS category_actor_id,
    (
        SELECT
            local
        FROM
            category c
        WHERE
            cf.category_id = c.id) AS category_local,
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
            actor_id
        FROM
            user_ u
        WHERE
            cm.user_id = u.id) AS user_actor_id,
    (
        SELECT
            local
        FROM
            user_ u
        WHERE
            cm.user_id = u.id) AS user_local,
    (
        SELECT
            name
        FROM
            user_ u
        WHERE
            cm.user_id = u.id) AS user_name,
    (
        SELECT
            avatar
        FROM
            user_ u
        WHERE
            cm.user_id = u.id), (
        SELECT
            actor_id
        FROM
            category c
        WHERE
            cm.category_id = c.id) AS category_actor_id,
    (
        SELECT
            local
        FROM
            category c
        WHERE
            cm.category_id = c.id) AS category_local,
    (
        SELECT
            name
        FROM
            category c
        WHERE
            cm.category_id = c.id) AS category_name
FROM
    category_user_ban cm;

-- post_view
DROP VIEW post_view;

DROP VIEW post_mview;

DROP MATERIALIZED VIEW post_aggregates_mview;

DROP VIEW post_aggregates_view;

-- regen post view
CREATE VIEW post_aggregates_view AS
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
            actor_id
        FROM
            user_
        WHERE
            p.creator_id = user_.id) AS creator_actor_id,
    (
        SELECT
            local
        FROM
            user_
        WHERE
            p.creator_id = user_.id) AS creator_local,
    (
        SELECT
            name
        FROM
            user_
        WHERE
            p.creator_id = user_.id) AS creator_name,
    (
        SELECT
            avatar
        FROM
            user_
        WHERE
            p.creator_id = user_.id) AS creator_avatar,
    (
        SELECT
            actor_id
        FROM
            category
        WHERE
            p.category_id = category.id) AS category_actor_id,
    (
        SELECT
            local
        FROM
            category
        WHERE
            p.category_id = category.id) AS category_local,
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
    hot_rank (coalesce(sum(pl.score), 0), (
            CASE WHEN (p.published < ('now'::timestamp - '1 month'::interval)) THEN
                p.published -- Prevents necro-bumps
            ELSE
                greatest (c.recent_comment_time, p.published)
            END)) AS hot_rank,
    (
        CASE WHEN (p.published < ('now'::timestamp - '1 month'::interval)) THEN
            p.published -- Prevents necro-bumps
        ELSE
            greatest (c.recent_comment_time, p.published)
        END) AS newest_activity_time
FROM
    post p
    LEFT JOIN post_like pl ON p.id = pl.post_id
    LEFT JOIN (
        SELECT
            post_id,
            max(published) AS recent_comment_time
        FROM
            comment
        GROUP BY
            1) c ON p.id = c.post_id
GROUP BY
    p.id,
    c.recent_comment_time;

CREATE MATERIALIZED VIEW post_aggregates_mview AS
SELECT
    *
FROM
    post_aggregates_view;

CREATE UNIQUE INDEX idx_post_aggregates_mview_id ON post_aggregates_mview (id);

CREATE VIEW post_view AS
with all_post AS (
    SELECT
        pa.*
    FROM
        post_aggregates_view pa
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

CREATE VIEW post_mview AS
with all_post AS (
    SELECT
        pa.*
    FROM
        post_aggregates_mview pa
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

-- reply_view, comment_view, user_mention
DROP VIEW reply_view;

DROP VIEW user_mention_view;

DROP VIEW user_mention_mview;

DROP VIEW comment_view;

DROP VIEW comment_mview;

DROP MATERIALIZED VIEW comment_aggregates_mview;

DROP VIEW comment_aggregates_view;

-- reply and comment view
CREATE VIEW comment_aggregates_view AS
SELECT
    c.*,
    (
        SELECT
            category_id
        FROM
            post p
        WHERE
            p.id = c.post_id), (
        SELECT
            co.actor_id
        FROM
            post p,
            category co
        WHERE
            p.id = c.post_id
            AND p.category_id = co.id) AS category_actor_id,
    (
        SELECT
            co.local
        FROM
            post p,
            category co
        WHERE
            p.id = c.post_id
            AND p.category_id = co.id) AS category_local,
    (
        SELECT
            co.name
        FROM
            post p,
            category co
        WHERE
            p.id = c.post_id
            AND p.category_id = co.id) AS category_name,
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
            actor_id
        FROM
            user_
        WHERE
            c.creator_id = user_.id) AS creator_actor_id,
    (
        SELECT
            local
        FROM
            user_
        WHERE
            c.creator_id = user_.id) AS creator_local,
    (
        SELECT
            name
        FROM
            user_
        WHERE
            c.creator_id = user_.id) AS creator_name,
    (
        SELECT
            avatar
        FROM
            user_
        WHERE
            c.creator_id = user_.id) AS creator_avatar,
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
        END) AS downvotes,
    hot_rank (coalesce(sum(cl.score), 0), c.published) AS hot_rank
FROM
    comment c
    LEFT JOIN comment_like cl ON c.id = cl.comment_id
GROUP BY
    c.id;

CREATE MATERIALIZED VIEW comment_aggregates_mview AS
SELECT
    *
FROM
    comment_aggregates_view;

CREATE UNIQUE INDEX idx_comment_aggregates_mview_id ON comment_aggregates_mview (id);

CREATE VIEW comment_view AS
with all_comment AS (
    SELECT
        ca.*
    FROM
        comment_aggregates_view ca
)
SELECT
    ac.*,
    u.id AS user_id,
    coalesce(cl.score, 0) AS my_vote,
    (
        SELECT
            cf.id::boolean
        FROM
            category_follower cf
        WHERE
            u.id = cf.user_id
            AND ac.category_id = cf.category_id) AS subscribed,
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
        NULL AS subscribed,
        NULL AS saved
    FROM
        all_comment ac;

CREATE VIEW comment_mview AS
with all_comment AS (
    SELECT
        ca.*
    FROM
        comment_aggregates_mview ca
)
SELECT
    ac.*,
    u.id AS user_id,
    coalesce(cl.score, 0) AS my_vote,
    (
        SELECT
            cf.id::boolean
        FROM
            category_follower cf
        WHERE
            u.id = cf.user_id
            AND ac.category_id = cf.category_id) AS subscribed,
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
        NULL AS subscribed,
        NULL AS saved
    FROM
        all_comment ac;

-- Do the reply_view referencing the comment_mview
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
    comment_mview cv,
    closereply
WHERE
    closereply.id = cv.id;

-- user mention
CREATE VIEW user_mention_view AS
SELECT
    c.id,
    um.id AS user_mention_id,
    c.creator_id,
    c.creator_actor_id,
    c.creator_local,
    c.post_id,
    c.parent_id,
    c.content,
    c.removed,
    um.read,
    c.published,
    c.updated,
    c.deleted,
    c.category_id,
    c.category_actor_id,
    c.category_local,
    c.category_name,
    c.banned,
    c.banned_from_category,
    c.creator_name,
    c.creator_avatar,
    c.score,
    c.upvotes,
    c.downvotes,
    c.hot_rank,
    c.user_id,
    c.my_vote,
    c.saved,
    um.recipient_id,
    (
        SELECT
            actor_id
        FROM
            user_ u
        WHERE
            u.id = um.recipient_id) AS recipient_actor_id,
    (
        SELECT
            local
        FROM
            user_ u
        WHERE
            u.id = um.recipient_id) AS recipient_local
FROM
    user_mention um,
    comment_view c
WHERE
    um.comment_id = c.id;

CREATE VIEW user_mention_mview AS
with all_comment AS (
    SELECT
        ca.*
    FROM
        comment_aggregates_mview ca
)
SELECT
    ac.id,
    um.id AS user_mention_id,
    ac.creator_id,
    ac.creator_actor_id,
    ac.creator_local,
    ac.post_id,
    ac.parent_id,
    ac.content,
    ac.removed,
    um.read,
    ac.published,
    ac.updated,
    ac.deleted,
    ac.category_id,
    ac.category_actor_id,
    ac.category_local,
    ac.category_name,
    ac.banned,
    ac.banned_from_category,
    ac.creator_name,
    ac.creator_avatar,
    ac.score,
    ac.upvotes,
    ac.downvotes,
    ac.hot_rank,
    u.id AS user_id,
    coalesce(cl.score, 0) AS my_vote,
    (
        SELECT
            cs.id::bool
        FROM
            comment_saved cs
        WHERE
            u.id = cs.user_id
            AND cs.comment_id = ac.id) AS saved,
    um.recipient_id,
    (
        SELECT
            actor_id
        FROM
            user_ u
        WHERE
            u.id = um.recipient_id) AS recipient_actor_id,
    (
        SELECT
            local
        FROM
            user_ u
        WHERE
            u.id = um.recipient_id) AS recipient_local
FROM
    user_ u
    CROSS JOIN all_comment ac
    LEFT JOIN comment_like cl ON u.id = cl.user_id
        AND ac.id = cl.comment_id
    LEFT JOIN user_mention um ON um.comment_id = ac.id
UNION ALL
SELECT
    ac.id,
    um.id AS user_mention_id,
    ac.creator_id,
    ac.creator_actor_id,
    ac.creator_local,
    ac.post_id,
    ac.parent_id,
    ac.content,
    ac.removed,
    um.read,
    ac.published,
    ac.updated,
    ac.deleted,
    ac.category_id,
    ac.category_actor_id,
    ac.category_local,
    ac.category_name,
    ac.banned,
    ac.banned_from_category,
    ac.creator_name,
    ac.creator_avatar,
    ac.score,
    ac.upvotes,
    ac.downvotes,
    ac.hot_rank,
    NULL AS user_id,
    NULL AS my_vote,
    NULL AS saved,
    um.recipient_id,
    (
        SELECT
            actor_id
        FROM
            user_ u
        WHERE
            u.id = um.recipient_id) AS recipient_actor_id,
    (
        SELECT
            local
        FROM
            user_ u
        WHERE
            u.id = um.recipient_id) AS recipient_local
FROM
    all_comment ac
    LEFT JOIN user_mention um ON um.comment_id = ac.id;

