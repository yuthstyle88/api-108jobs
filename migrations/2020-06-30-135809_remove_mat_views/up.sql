-- Drop the mviews
DROP VIEW post_mview;

DROP MATERIALIZED VIEW user_mview;

DROP VIEW category_mview;


DROP VIEW user_mention_mview;

DROP VIEW reply_view;

DROP VIEW comment_mview;

DROP MATERIALIZED VIEW post_aggregates_mview;

DROP MATERIALIZED VIEW category_aggregates_mview;

DROP MATERIALIZED VIEW comment_aggregates_mview;

-- User
DROP VIEW user_view;

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
    coalesce(pd.posts, 0) AS number_of_posts,
    coalesce(pd.score, 0) AS post_score,
    coalesce(cd.comments, 0) AS number_of_comments,
    coalesce(cd.score, 0) AS comment_score
FROM
    user_ u
    LEFT JOIN (
        SELECT
            p.creator_id AS creator_id,
            count(DISTINCT p.id) AS posts,
            sum(pl.score) AS score
        FROM
            post p
            JOIN post_like pl ON p.id = pl.post_id
        GROUP BY
            p.creator_id) pd ON u.id = pd.creator_id
    LEFT JOIN (
        SELECT
            c.creator_id,
            count(DISTINCT c.id) AS comments,
            sum(cl.score) AS score
        FROM
            comment c
            JOIN comment_like cl ON c.id = cl.comment_id
        GROUP BY
            c.creator_id) cd ON u.id = cd.creator_id;

CREATE TABLE user_fast AS
SELECT
    *
FROM
    user_view;

ALTER TABLE user_fast
    ADD PRIMARY KEY (id);

DROP TRIGGER refresh_user ON user_;

CREATE TRIGGER refresh_user
    AFTER INSERT OR UPDATE OR DELETE ON user_
    FOR EACH ROW
    EXECUTE PROCEDURE refresh_user ();

-- Sample insert
-- insert into user_(name, password_encrypted) values ('test_name', 'bleh');
-- Sample delete
-- delete from user_ where name like 'test_name';
-- Sample update
-- update user_ set avatar = 'hai'  where name like 'test_name';
CREATE OR REPLACE FUNCTION refresh_user ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'DELETE') THEN
        DELETE FROM user_fast
        WHERE id = OLD.id;
    ELSIF (TG_OP = 'UPDATE') THEN
        DELETE FROM user_fast
        WHERE id = OLD.id;
        INSERT INTO user_fast
        SELECT
            *
        FROM
            user_view
        WHERE
            id = NEW.id;
        -- Refresh post_fast, cause of user info changes
        DELETE FROM post_aggregates_fast
        WHERE creator_id = NEW.id;
        INSERT INTO post_aggregates_fast
        SELECT
            *
        FROM
            post_aggregates_view
        WHERE
            creator_id = NEW.id;
        DELETE FROM comment_aggregates_fast
        WHERE creator_id = NEW.id;
        INSERT INTO comment_aggregates_fast
        SELECT
            *
        FROM
            comment_aggregates_view
        WHERE
            creator_id = NEW.id;
    ELSIF (TG_OP = 'INSERT') THEN
        INSERT INTO user_fast
        SELECT
            *
        FROM
            user_view
        WHERE
            id = NEW.id;
    END IF;
    RETURN NULL;
END
$$;

-- Post
-- Redoing the views : Credit eiknat
DROP VIEW post_view;

DROP VIEW post_aggregates_view;

CREATE VIEW post_aggregates_view AS
SELECT
    p.*,
    -- creator details
    u.actor_id AS creator_actor_id,
    u."local" AS creator_local,
    u."name" AS creator_name,
    u.avatar AS creator_avatar,
    u.banned AS banned,
    cb.id::bool AS banned_from_category,
    -- category details
    c.actor_id AS category_actor_id,
    c."local" AS category_local,
    c."name" AS category_name,
    c.removed AS category_removed,
    c.deleted AS category_deleted,
    c.self_promotion AS category_self_promotion,
    -- post score data/comment count
    coalesce(ct.comments, 0) AS number_of_comments,
    coalesce(pl.score, 0) AS score,
    coalesce(pl.upvotes, 0) AS upvotes,
    coalesce(pl.downvotes, 0) AS downvotes,
    hot_rank (coalesce(pl.score, 0), (
            CASE WHEN (p.published < ('now'::timestamp - '1 month'::interval)) THEN
                p.published
            ELSE
                greatest (ct.recent_comment_time, p.published)
            END)) AS hot_rank,
    (
        CASE WHEN (p.published < ('now'::timestamp - '1 month'::interval)) THEN
            p.published
        ELSE
            greatest (ct.recent_comment_time, p.published)
        END) AS newest_activity_time
FROM
    post p
    LEFT JOIN user_ u ON p.creator_id = u.id
    LEFT JOIN category_user_ban cb ON p.creator_id = cb.user_id
        AND p.category_id = cb.category_id
    LEFT JOIN category c ON p.category_id = c.id
    LEFT JOIN (
        SELECT
            post_id,
            count(*) AS comments,
            max(published) AS recent_comment_time
        FROM
            comment
        GROUP BY
            post_id) ct ON ct.post_id = p.id
    LEFT JOIN (
        SELECT
            post_id,
            sum(score) AS score,
            sum(score) FILTER (WHERE score = 1) AS upvotes,
            - sum(score) FILTER (WHERE score = -1) AS downvotes
        FROM
            post_like
        GROUP BY
            post_id) pl ON pl.post_id = p.id
ORDER BY
    p.id;

CREATE VIEW post_view AS
SELECT
    pav.*,
    us.id AS user_id,
    us.user_vote AS my_vote,
    us.is_subbed::bool AS subscribed,
    us.is_read::bool AS read,
    us.is_saved::bool AS saved
FROM
    post_aggregates_view pav
    CROSS JOIN LATERAL (
        SELECT
            u.id,
            coalesce(cf.category_id, 0) AS is_subbed,
            coalesce(pr.post_id, 0) AS is_read,
            coalesce(ps.post_id, 0) AS is_saved,
            coalesce(pl.score, 0) AS user_vote
        FROM
            user_ u
            LEFT JOIN category_user_ban cb ON u.id = cb.user_id
                AND cb.category_id = pav.category_id
        LEFT JOIN category_follower cf ON u.id = cf.user_id
            AND cf.category_id = pav.category_id
    LEFT JOIN post_read pr ON u.id = pr.user_id
        AND pr.post_id = pav.id
    LEFT JOIN post_saved ps ON u.id = ps.user_id
        AND ps.post_id = pav.id
    LEFT JOIN post_like pl ON u.id = pl.user_id
        AND pav.id = pl.post_id) AS us
UNION ALL
SELECT
    pav.*,
    NULL AS user_id,
    NULL AS my_vote,
    NULL AS subscribed,
    NULL AS read,
    NULL AS saved
FROM
    post_aggregates_view pav;

-- The post fast table
CREATE TABLE post_aggregates_fast AS
SELECT
    *
FROM
    post_aggregates_view;

ALTER TABLE post_aggregates_fast
    ADD PRIMARY KEY (id);

-- For the hot rank resorting
CREATE INDEX idx_post_aggregates_fast_hot_rank_published ON post_aggregates_fast (hot_rank DESC, published DESC);

CREATE VIEW post_fast_view AS
SELECT
    pav.*,
    us.id AS user_id,
    us.user_vote AS my_vote,
    us.is_subbed::bool AS subscribed,
    us.is_read::bool AS read,
    us.is_saved::bool AS saved
FROM
    post_aggregates_fast pav
    CROSS JOIN LATERAL (
        SELECT
            u.id,
            coalesce(cf.category_id, 0) AS is_subbed,
            coalesce(pr.post_id, 0) AS is_read,
            coalesce(ps.post_id, 0) AS is_saved,
            coalesce(pl.score, 0) AS user_vote
        FROM
            user_ u
            LEFT JOIN category_user_ban cb ON u.id = cb.user_id
                AND cb.category_id = pav.category_id
        LEFT JOIN category_follower cf ON u.id = cf.user_id
            AND cf.category_id = pav.category_id
    LEFT JOIN post_read pr ON u.id = pr.user_id
        AND pr.post_id = pav.id
    LEFT JOIN post_saved ps ON u.id = ps.user_id
        AND ps.post_id = pav.id
    LEFT JOIN post_like pl ON u.id = pl.user_id
        AND pav.id = pl.post_id) AS us
UNION ALL
SELECT
    pav.*,
    NULL AS user_id,
    NULL AS my_vote,
    NULL AS subscribed,
    NULL AS read,
    NULL AS saved
FROM
    post_aggregates_fast pav;

DROP TRIGGER refresh_post ON post;

CREATE TRIGGER refresh_post
    AFTER INSERT OR UPDATE OR DELETE ON post
    FOR EACH ROW
    EXECUTE PROCEDURE refresh_post ();

-- Sample select
-- select id, name from post_fast_view where name like 'test_post' and user_id is null;
-- Sample insert
-- insert into post(name, creator_id, category_id) values ('test_post', 2, 2);
-- Sample delete
-- delete from post where name like 'test_post';
-- Sample update
-- update post set category_id = 4  where name like 'test_post';
CREATE OR REPLACE FUNCTION refresh_post ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'DELETE') THEN
        DELETE FROM post_aggregates_fast
        WHERE id = OLD.id;
        -- Update category number of posts
        UPDATE
            category_aggregates_fast
        SET
            number_of_posts = number_of_posts - 1
        WHERE
            id = OLD.category_id;
    ELSIF (TG_OP = 'UPDATE') THEN
        DELETE FROM post_aggregates_fast
        WHERE id = OLD.id;
        INSERT INTO post_aggregates_fast
        SELECT
            *
        FROM
            post_aggregates_view
        WHERE
            id = NEW.id;
    ELSIF (TG_OP = 'INSERT') THEN
        INSERT INTO post_aggregates_fast
        SELECT
            *
        FROM
            post_aggregates_view
        WHERE
            id = NEW.id;
        -- Update that users number of posts, post score
        DELETE FROM user_fast
        WHERE id = NEW.creator_id;
        INSERT INTO user_fast
        SELECT
            *
        FROM
            user_view
        WHERE
            id = NEW.creator_id;
        -- Update category number of posts
        UPDATE
            category_aggregates_fast
        SET
            number_of_posts = number_of_posts + 1
        WHERE
            id = NEW.category_id;
        -- Update the hot rank on the post table
        -- TODO this might not correctly update it, using a 1 week interval
        UPDATE
            post_aggregates_fast AS paf
        SET
            hot_rank = pav.hot_rank
        FROM
            post_aggregates_view AS pav
        WHERE
            paf.id = pav.id
            AND (pav.published > ('now'::timestamp - '1 week'::interval));
    END IF;
    RETURN NULL;
END
$$;

-- Category
-- Redoing the views : Credit eiknat
DROP VIEW category_moderator_view;

DROP VIEW category_follower_view;

DROP VIEW category_user_ban_view;

DROP VIEW category_view;

DROP VIEW category_aggregates_view;

CREATE VIEW category_aggregates_view AS
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
    u.actor_id AS creator_actor_id,
    u.local AS creator_local,
    u.name AS creator_name,
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

CREATE VIEW category_moderator_view AS
SELECT
    cm.*,
    u.actor_id AS user_actor_id,
    u.local AS user_local,
    u.name AS user_name,
    u.avatar AS avatar,
    c.actor_id AS category_actor_id,
    c.local AS category_local,
    c.name AS category_name
FROM
    category_moderator cm
    LEFT JOIN user_ u ON cm.user_id = u.id
    LEFT JOIN category c ON cm.category_id = c.id;

CREATE VIEW category_follower_view AS
SELECT
    cf.*,
    u.actor_id AS user_actor_id,
    u.local AS user_local,
    u.name AS user_name,
    u.avatar AS avatar,
    c.actor_id AS category_actor_id,
    c.local AS category_local,
    c.name AS category_name
FROM
    category_follower cf
    LEFT JOIN user_ u ON cf.user_id = u.id
    LEFT JOIN category c ON cf.category_id = c.id;

CREATE VIEW category_user_ban_view AS
SELECT
    cb.*,
    u.actor_id AS user_actor_id,
    u.local AS user_local,
    u.name AS user_name,
    u.avatar AS avatar,
    c.actor_id AS category_actor_id,
    c.local AS category_local,
    c.name AS category_name
FROM
    category_user_ban cb
    LEFT JOIN user_ u ON cb.user_id = u.id
    LEFT JOIN category c ON cb.category_id = c.id;

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

DROP TRIGGER refresh_category ON category;

CREATE TRIGGER refresh_category
    AFTER INSERT OR UPDATE OR DELETE ON category
    FOR EACH ROW
    EXECUTE PROCEDURE refresh_category ();

-- Sample select
-- select * from category_fast_view where name like 'test_category_name' and user_id is null;
-- Sample insert
-- insert into category(name, title, category_id, creator_id) values ('test_category_name', 'test_category_title', 1, 2);
-- Sample delete
-- delete from category where name like 'test_category_name';
-- Sample update
-- update category set title = 'test_category_title_2'  where name like 'test_category_name';
CREATE OR REPLACE FUNCTION refresh_category ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'DELETE') THEN
        DELETE FROM category_aggregates_fast
        WHERE id = OLD.id;
    ELSIF (TG_OP = 'UPDATE') THEN
        DELETE FROM category_aggregates_fast
        WHERE id = OLD.id;
        INSERT INTO category_aggregates_fast
        SELECT
            *
        FROM
            category_aggregates_view
        WHERE
            id = NEW.id;
        -- Update user view due to owner changes
        DELETE FROM user_fast
        WHERE id = NEW.creator_id;
        INSERT INTO user_fast
        SELECT
            *
        FROM
            user_view
        WHERE
            id = NEW.creator_id;
        -- Update post view due to category changes
        DELETE FROM post_aggregates_fast
        WHERE category_id = NEW.id;
        INSERT INTO post_aggregates_fast
        SELECT
            *
        FROM
            post_aggregates_view
        WHERE
            category_id = NEW.id;
        -- TODO make sure this shows up in the users page ?
    ELSIF (TG_OP = 'INSERT') THEN
        INSERT INTO category_aggregates_fast
        SELECT
            *
        FROM
            category_aggregates_view
        WHERE
            id = NEW.id;
    END IF;
    RETURN NULL;
END
$$;

-- Comment
DROP VIEW user_mention_view;

DROP VIEW comment_view;

DROP VIEW comment_aggregates_view;

CREATE VIEW comment_aggregates_view AS
SELECT
    ct.*,
    -- category details
    p.category_id,
    c.actor_id AS category_actor_id,
    c."local" AS category_local,
    c."name" AS category_name,
    -- creator details
    u.banned AS banned,
    coalesce(cb.id, 0)::bool AS banned_from_category,
    u.actor_id AS creator_actor_id,
    u.local AS creator_local,
    u.name AS creator_name,
    u.avatar AS creator_avatar,
    -- score details
    coalesce(cl.total, 0) AS score,
    coalesce(cl.up, 0) AS upvotes,
    coalesce(cl.down, 0) AS downvotes,
    hot_rank (coalesce(cl.total, 0), ct.published) AS hot_rank
FROM
    comment ct
    LEFT JOIN post p ON ct.post_id = p.id
    LEFT JOIN category c ON p.category_id = c.id
    LEFT JOIN user_ u ON ct.creator_id = u.id
    LEFT JOIN category_user_ban cb ON ct.creator_id = cb.user_id
        AND p.id = ct.post_id
        AND p.category_id = cb.category_id
    LEFT JOIN (
        SELECT
            l.comment_id AS id,
            sum(l.score) AS total,
            count(
                CASE WHEN l.score = 1 THEN
                    1
                ELSE
                    NULL
                END) AS up,
            count(
                CASE WHEN l.score = -1 THEN
                    1
                ELSE
                    NULL
                END) AS down
        FROM
            comment_like l
        GROUP BY
            comment_id) AS cl ON cl.id = ct.id;

CREATE OR REPLACE VIEW comment_view AS (
    SELECT
        cav.*,
        us.user_id AS user_id,
        us.my_vote AS my_vote,
        us.is_subbed::bool AS subscribed,
        us.is_saved::bool AS saved
    FROM
        comment_aggregates_view cav
    CROSS JOIN LATERAL (
        SELECT
            u.id AS user_id,
            coalesce(cl.score, 0) AS my_vote,
            coalesce(cf.id, 0) AS is_subbed,
            coalesce(cs.id, 0) AS is_saved
        FROM
            user_ u
            LEFT JOIN comment_like cl ON u.id = cl.user_id
                AND cav.id = cl.comment_id
        LEFT JOIN comment_saved cs ON u.id = cs.user_id
            AND cs.comment_id = cav.id
    LEFT JOIN category_follower cf ON u.id = cf.user_id
        AND cav.category_id = cf.category_id) AS us
UNION ALL
SELECT
    cav.*,
    NULL AS user_id,
    NULL AS my_vote,
    NULL AS subscribed,
    NULL AS saved
FROM
    comment_aggregates_view cav);

-- The fast view
CREATE TABLE comment_aggregates_fast AS
SELECT
    *
FROM
    comment_aggregates_view;

ALTER TABLE comment_aggregates_fast
    ADD PRIMARY KEY (id);

CREATE VIEW comment_fast_view AS
SELECT
    cav.*,
    us.user_id AS user_id,
    us.my_vote AS my_vote,
    us.is_subbed::bool AS subscribed,
    us.is_saved::bool AS saved
FROM
    comment_aggregates_fast cav
    CROSS JOIN LATERAL (
        SELECT
            u.id AS user_id,
            coalesce(cl.score, 0) AS my_vote,
            coalesce(cf.id, 0) AS is_subbed,
            coalesce(cs.id, 0) AS is_saved
        FROM
            user_ u
            LEFT JOIN comment_like cl ON u.id = cl.user_id
                AND cav.id = cl.comment_id
        LEFT JOIN comment_saved cs ON u.id = cs.user_id
            AND cs.comment_id = cav.id
    LEFT JOIN category_follower cf ON u.id = cf.user_id
        AND cav.category_id = cf.category_id) AS us
UNION ALL
SELECT
    cav.*,
    NULL AS user_id,
    NULL AS my_vote,
    NULL AS subscribed,
    NULL AS saved
FROM
    comment_aggregates_fast cav;

-- Do the reply_view referencing the comment_fast_view
CREATE VIEW reply_fast_view AS
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
    comment_fast_view cv,
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

CREATE VIEW user_mention_fast_view AS
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
    CROSS JOIN (
        SELECT
            ca.*
        FROM
            comment_aggregates_fast ca) ac
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
    comment_aggregates_fast ac
    LEFT JOIN user_mention um ON um.comment_id = ac.id;

DROP TRIGGER refresh_comment ON comment;

CREATE TRIGGER refresh_comment
    AFTER INSERT OR UPDATE OR DELETE ON comment
    FOR EACH ROW
    EXECUTE PROCEDURE refresh_comment ();

-- Sample select
-- select * from comment_fast_view where content = 'test_comment' and user_id is null;
-- Sample insert
-- insert into comment(creator_id, post_id, content) values (2, 2, 'test_comment');
-- Sample delete
-- delete from comment where content like 'test_comment';
-- Sample update
-- update comment set removed = true where content like 'test_comment';
CREATE OR REPLACE FUNCTION refresh_comment ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'DELETE') THEN
        DELETE FROM comment_aggregates_fast
        WHERE id = OLD.id;
        -- Update category number of comments
        UPDATE
            category_aggregates_fast AS caf
        SET
            number_of_comments = number_of_comments - 1
        FROM
            post AS p
        WHERE
            caf.id = p.category_id
            AND p.id = OLD.post_id;
    ELSIF (TG_OP = 'UPDATE') THEN
        DELETE FROM comment_aggregates_fast
        WHERE id = OLD.id;
        INSERT INTO comment_aggregates_fast
        SELECT
            *
        FROM
            comment_aggregates_view
        WHERE
            id = NEW.id;
    ELSIF (TG_OP = 'INSERT') THEN
        INSERT INTO comment_aggregates_fast
        SELECT
            *
        FROM
            comment_aggregates_view
        WHERE
            id = NEW.id;
        -- Update user view due to comment count
        UPDATE
            user_fast
        SET
            number_of_comments = number_of_comments + 1
        WHERE
            id = NEW.creator_id;
        -- Update post view due to comment count, new comment activity time, but only on new posts
        -- TODO this could be done more efficiently
        DELETE FROM post_aggregates_fast
        WHERE id = NEW.post_id;
        INSERT INTO post_aggregates_fast
        SELECT
            *
        FROM
            post_aggregates_view
        WHERE
            id = NEW.post_id;
        -- Force the hot rank as zero on week-older posts
        UPDATE
            post_aggregates_fast AS paf
        SET
            hot_rank = 0
        WHERE
            paf.id = NEW.post_id
            AND (paf.published < ('now'::timestamp - '1 week'::interval));
        -- Update category number of comments
        UPDATE
            category_aggregates_fast AS caf
        SET
            number_of_comments = number_of_comments + 1
        FROM
            post AS p
        WHERE
            caf.id = p.category_id
            AND p.id = NEW.post_id;
    END IF;
    RETURN NULL;
END
$$;

-- post_like
-- select id, score, my_vote from post_fast_view where id = 29 and user_id = 4;
-- Sample insert
-- insert into post_like(user_id, post_id, score) values (4, 29, 1);
-- Sample delete
-- delete from post_like where user_id = 4 and post_id = 29;
-- Sample update
-- update post_like set score = -1 where user_id = 4 and post_id = 29;
-- TODO test this a LOT
CREATE OR REPLACE FUNCTION refresh_post_like ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'DELETE') THEN
        UPDATE
            post_aggregates_fast
        SET
            score = CASE WHEN (OLD.score = 1) THEN
                score - 1
            ELSE
                score + 1
            END,
            upvotes = CASE WHEN (OLD.score = 1) THEN
                upvotes - 1
            ELSE
                upvotes
            END,
            downvotes = CASE WHEN (OLD.score = -1) THEN
                downvotes - 1
            ELSE
                downvotes
            END
        WHERE
            id = OLD.post_id;
    ELSIF (TG_OP = 'INSERT') THEN
        UPDATE
            post_aggregates_fast
        SET
            score = CASE WHEN (NEW.score = 1) THEN
                score + 1
            ELSE
                score - 1
            END,
            upvotes = CASE WHEN (NEW.score = 1) THEN
                upvotes + 1
            ELSE
                upvotes
            END,
            downvotes = CASE WHEN (NEW.score = -1) THEN
                downvotes + 1
            ELSE
                downvotes
            END
        WHERE
            id = NEW.post_id;
    END IF;
    RETURN NULL;
END
$$;

DROP TRIGGER refresh_post_like ON post_like;

CREATE TRIGGER refresh_post_like
    AFTER INSERT OR DELETE ON post_like
    FOR EACH ROW
    EXECUTE PROCEDURE refresh_post_like ();

-- comment_like
-- select id, score, my_vote from comment_fast_view where id = 29 and user_id = 4;
-- Sample insert
-- insert into comment_like(user_id, comment_id, post_id, score) values (4, 29, 51, 1);
-- Sample delete
-- delete from comment_like where user_id = 4 and comment_id = 29;
-- Sample update
-- update comment_like set score = -1 where user_id = 4 and comment_id = 29;
CREATE OR REPLACE FUNCTION refresh_comment_like ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    -- TODO possibly select from comment_fast to get previous scores, instead of re-fetching the views?
    IF (TG_OP = 'DELETE') THEN
        UPDATE
            comment_aggregates_fast
        SET
            score = CASE WHEN (OLD.score = 1) THEN
                score - 1
            ELSE
                score + 1
            END,
            upvotes = CASE WHEN (OLD.score = 1) THEN
                upvotes - 1
            ELSE
                upvotes
            END,
            downvotes = CASE WHEN (OLD.score = -1) THEN
                downvotes - 1
            ELSE
                downvotes
            END
        WHERE
            id = OLD.comment_id;
    ELSIF (TG_OP = 'INSERT') THEN
        UPDATE
            comment_aggregates_fast
        SET
            score = CASE WHEN (NEW.score = 1) THEN
                score + 1
            ELSE
                score - 1
            END,
            upvotes = CASE WHEN (NEW.score = 1) THEN
                upvotes + 1
            ELSE
                upvotes
            END,
            downvotes = CASE WHEN (NEW.score = -1) THEN
                downvotes + 1
            ELSE
                downvotes
            END
        WHERE
            id = NEW.comment_id;
    END IF;
    RETURN NULL;
END
$$;

DROP TRIGGER refresh_comment_like ON comment_like;

CREATE TRIGGER refresh_comment_like
    AFTER INSERT OR DELETE ON comment_like
    FOR EACH ROW
    EXECUTE PROCEDURE refresh_comment_like ();

-- Category user ban
DROP TRIGGER refresh_category_user_ban ON category_user_ban;

CREATE TRIGGER refresh_category_user_ban
    AFTER INSERT OR DELETE -- Note this is missing after update
    ON category_user_ban
    FOR EACH ROW
    EXECUTE PROCEDURE refresh_category_user_ban ();

-- select creator_name, banned_from_category from comment_fast_view where user_id = 4 and content = 'test_before_ban';
-- select creator_name, banned_from_category, category_id from comment_aggregates_fast where content = 'test_before_ban';
-- Sample insert
-- insert into comment(creator_id, post_id, content) values (1198, 341, 'test_before_ban');
-- insert into category_user_ban(category_id, user_id) values (2, 1198);
-- Sample delete
-- delete from category_user_ban where user_id = 1198 and category_id = 2;
-- delete from comment where content = 'test_before_ban';
-- update comment_aggregates_fast set banned_from_category = false where creator_id = 1198 and category_id = 2;
CREATE OR REPLACE FUNCTION refresh_category_user_ban ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    -- TODO possibly select from comment_fast to get previous scores, instead of re-fetching the views?
    IF (TG_OP = 'DELETE') THEN
        UPDATE
            comment_aggregates_fast
        SET
            banned_from_category = FALSE
        WHERE
            creator_id = OLD.user_id
            AND category_id = OLD.category_id;
        UPDATE
            post_aggregates_fast
        SET
            banned_from_category = FALSE
        WHERE
            creator_id = OLD.user_id
            AND category_id = OLD.category_id;
    ELSIF (TG_OP = 'INSERT') THEN
        UPDATE
            comment_aggregates_fast
        SET
            banned_from_category = TRUE
        WHERE
            creator_id = NEW.user_id
            AND category_id = NEW.category_id;
        UPDATE
            post_aggregates_fast
        SET
            banned_from_category = TRUE
        WHERE
            creator_id = NEW.user_id
            AND category_id = NEW.category_id;
    END IF;
    RETURN NULL;
END
$$;

-- Category follower
DROP TRIGGER refresh_category_follower ON category_follower;

CREATE TRIGGER refresh_category_follower
    AFTER INSERT OR DELETE -- Note this is missing after update
    ON category_follower
    FOR EACH ROW
    EXECUTE PROCEDURE refresh_category_follower ();

CREATE OR REPLACE FUNCTION refresh_category_follower ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'DELETE') THEN
        UPDATE
            category_aggregates_fast
        SET
            number_of_subscribers = number_of_subscribers - 1
        WHERE
            id = OLD.category_id;
    ELSIF (TG_OP = 'INSERT') THEN
        UPDATE
            category_aggregates_fast
        SET
            number_of_subscribers = number_of_subscribers + 1
        WHERE
            id = NEW.category_id;
    END IF;
    RETURN NULL;
END
$$;

