-- Add category aggregates
CREATE TABLE category_aggregates (
    id serial PRIMARY KEY,
    category_id int REFERENCES category ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    subscribers bigint NOT NULL DEFAULT 0,
    posts bigint NOT NULL DEFAULT 0,
    comments bigint NOT NULL DEFAULT 0,
    published timestamp NOT NULL DEFAULT now(),
    UNIQUE (category_id)
);

INSERT INTO category_aggregates (category_id, subscribers, posts, comments, published)
SELECT
    c.id,
    coalesce(cf.subs, 0) AS subscribers,
    coalesce(cd.posts, 0) AS posts,
    coalesce(cd.comments, 0) AS comments,
    c.published
FROM
    category c
    LEFT JOIN (
        SELECT
            p.category_id,
            count(DISTINCT p.id) AS posts,
            count(DISTINCT ct.id) AS comments
        FROM
            post p
            LEFT JOIN comment ct ON p.id = ct.post_id
        GROUP BY
            p.category_id) cd ON cd.category_id = c.id
    LEFT JOIN (
        SELECT
            category_follower.category_id,
            count(*) AS subs
        FROM
            category_follower
        GROUP BY
            category_follower.category_id) cf ON cf.category_id = c.id;

-- Add category aggregate triggers
-- initial category add
CREATE FUNCTION category_aggregates_category ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        INSERT INTO category_aggregates (category_id)
            VALUES (NEW.id);
    ELSIF (TG_OP = 'DELETE') THEN
        DELETE FROM category_aggregates
        WHERE category_id = OLD.id;
    END IF;
    RETURN NULL;
END
$$;

CREATE TRIGGER category_aggregates_category
    AFTER INSERT OR DELETE ON category
    FOR EACH ROW
    EXECUTE PROCEDURE category_aggregates_category ();

-- post count
CREATE FUNCTION category_aggregates_post_count ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        UPDATE
            category_aggregates
        SET
            posts = posts + 1
        WHERE
            category_id = NEW.category_id;
    ELSIF (TG_OP = 'DELETE') THEN
        UPDATE
            category_aggregates
        SET
            posts = posts - 1
        WHERE
            category_id = OLD.category_id;
        -- Update the counts if the post got deleted
        UPDATE
            category_aggregates ca
        SET
            posts = coalesce(cd.posts, 0),
            comments = coalesce(cd.comments, 0)
        FROM (
            SELECT
                c.id,
                count(DISTINCT p.id) AS posts,
                count(DISTINCT ct.id) AS comments
            FROM
                category c
            LEFT JOIN post p ON c.id = p.category_id
            LEFT JOIN comment ct ON p.id = ct.post_id
        GROUP BY
            c.id) cd
    WHERE
        ca.category_id = OLD.category_id;
    END IF;
    RETURN NULL;
END
$$;

CREATE TRIGGER category_aggregates_post_count
    AFTER INSERT OR DELETE ON post
    FOR EACH ROW
    EXECUTE PROCEDURE category_aggregates_post_count ();

-- comment count
CREATE FUNCTION category_aggregates_comment_count ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        UPDATE
            category_aggregates ca
        SET
            comments = comments + 1
        FROM
            comment c,
            post p
        WHERE
            p.id = c.post_id
            AND p.id = NEW.post_id
            AND ca.category_id = p.category_id;
    ELSIF (TG_OP = 'DELETE') THEN
        UPDATE
            category_aggregates ca
        SET
            comments = comments - 1
        FROM
            comment c,
            post p
        WHERE
            p.id = c.post_id
            AND p.id = OLD.post_id
            AND ca.category_id = p.category_id;
    END IF;
    RETURN NULL;
END
$$;

CREATE TRIGGER category_aggregates_comment_count
    AFTER INSERT OR DELETE ON comment
    FOR EACH ROW
    EXECUTE PROCEDURE category_aggregates_comment_count ();

-- subscriber count
CREATE FUNCTION category_aggregates_subscriber_count ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        UPDATE
            category_aggregates
        SET
            subscribers = subscribers + 1
        WHERE
            category_id = NEW.category_id;
    ELSIF (TG_OP = 'DELETE') THEN
        UPDATE
            category_aggregates
        SET
            subscribers = subscribers - 1
        WHERE
            category_id = OLD.category_id;
    END IF;
    RETURN NULL;
END
$$;

CREATE TRIGGER category_aggregates_subscriber_count
    AFTER INSERT OR DELETE ON category_follower
    FOR EACH ROW
    EXECUTE PROCEDURE category_aggregates_subscriber_count ();

