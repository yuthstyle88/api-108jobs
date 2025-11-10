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
                AND p.id = co.post_id) AS number_of_comments
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
            comment) AS number_of_comments
FROM
    site s;

