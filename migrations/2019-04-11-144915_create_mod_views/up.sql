CREATE VIEW mod_remove_post_view AS
SELECT
    mrp.*,
    (
        SELECT
            name
        FROM
            user_ u
        WHERE
            mrp.mod_user_id = u.id) AS mod_user_name,
    (
        SELECT
            name
        FROM
            post p
        WHERE
            mrp.post_id = p.id) AS post_name,
    (
        SELECT
            c.id
        FROM
            post p,
            category c
        WHERE
            mrp.post_id = p.id
            AND p.category_id = c.id) AS category_id,
    (
        SELECT
            c.name
        FROM
            post p,
            category c
        WHERE
            mrp.post_id = p.id
            AND p.category_id = c.id) AS category_name
FROM
    mod_remove_post mrp;

CREATE VIEW mod_lock_post_view AS
SELECT
    mlp.*,
    (
        SELECT
            name
        FROM
            user_ u
        WHERE
            mlp.mod_user_id = u.id) AS mod_user_name,
    (
        SELECT
            name
        FROM
            post p
        WHERE
            mlp.post_id = p.id) AS post_name,
    (
        SELECT
            c.id
        FROM
            post p,
            category c
        WHERE
            mlp.post_id = p.id
            AND p.category_id = c.id) AS category_id,
    (
        SELECT
            c.name
        FROM
            post p,
            category c
        WHERE
            mlp.post_id = p.id
            AND p.category_id = c.id) AS category_name
FROM
    mod_lock_post mlp;

CREATE VIEW mod_remove_comment_view AS
SELECT
    mrc.*,
    (
        SELECT
            name
        FROM
            user_ u
        WHERE
            mrc.mod_user_id = u.id) AS mod_user_name,
    (
        SELECT
            c.id
        FROM
            comment c
        WHERE
            mrc.comment_id = c.id) AS comment_user_id,
    (
        SELECT
            name
        FROM
            user_ u,
            comment c
        WHERE
            mrc.comment_id = c.id
            AND u.id = c.creator_id) AS comment_user_name,
    (
        SELECT
            content
        FROM
            comment c
        WHERE
            mrc.comment_id = c.id) AS comment_content,
    (
        SELECT
            p.id
        FROM
            post p,
            comment c
        WHERE
            mrc.comment_id = c.id
            AND c.post_id = p.id) AS post_id,
    (
        SELECT
            p.name
        FROM
            post p,
            comment c
        WHERE
            mrc.comment_id = c.id
            AND c.post_id = p.id) AS post_name,
    (
        SELECT
            co.id
        FROM
            comment c,
            post p,
            category co
        WHERE
            mrc.comment_id = c.id
            AND c.post_id = p.id
            AND p.category_id = co.id) AS category_id,
    (
        SELECT
            co.name
        FROM
            comment c,
            post p,
            category co
        WHERE
            mrc.comment_id = c.id
            AND c.post_id = p.id
            AND p.category_id = co.id) AS category_name
FROM
    mod_remove_comment mrc;

CREATE VIEW mod_remove_category_view AS
SELECT
    mrc.*,
    (
        SELECT
            name
        FROM
            user_ u
        WHERE
            mrc.mod_user_id = u.id) AS mod_user_name,
    (
        SELECT
            c.name
        FROM
            category c
        WHERE
            mrc.category_id = c.id) AS category_name
FROM
    mod_remove_category mrc;

CREATE VIEW mod_ban_from_category_view AS
SELECT
    mb.*,
    (
        SELECT
            name
        FROM
            user_ u
        WHERE
            mb.mod_user_id = u.id) AS mod_user_name,
    (
        SELECT
            name
        FROM
            user_ u
        WHERE
            mb.other_user_id = u.id) AS other_user_name,
    (
        SELECT
            name
        FROM
            category c
        WHERE
            mb.category_id = c.id) AS category_name
FROM
    mod_ban_from_category mb;

CREATE VIEW mod_ban_view AS
SELECT
    mb.*,
    (
        SELECT
            name
        FROM
            user_ u
        WHERE
            mb.mod_user_id = u.id) AS mod_user_name,
    (
        SELECT
            name
        FROM
            user_ u
        WHERE
            mb.other_user_id = u.id) AS other_user_name
FROM
    mod_ban mb;

CREATE VIEW mod_add_category_view AS
SELECT
    ma.*,
    (
        SELECT
            name
        FROM
            user_ u
        WHERE
            ma.mod_user_id = u.id) AS mod_user_name,
    (
        SELECT
            name
        FROM
            user_ u
        WHERE
            ma.other_user_id = u.id) AS other_user_name,
    (
        SELECT
            name
        FROM
            category c
        WHERE
            ma.category_id = c.id) AS category_name
FROM
    mod_add_category ma;

CREATE VIEW mod_add_view AS
SELECT
    ma.*,
    (
        SELECT
            name
        FROM
            user_ u
        WHERE
            ma.mod_user_id = u.id) AS mod_user_name,
    (
        SELECT
            name
        FROM
            user_ u
        WHERE
            ma.other_user_id = u.id) AS other_user_name
FROM
    mod_add ma;

