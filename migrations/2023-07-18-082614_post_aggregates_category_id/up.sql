-- Your SQL goes here
ALTER TABLE post_aggregates
    ADD COLUMN category_id integer REFERENCES category ON UPDATE CASCADE ON DELETE CASCADE,
    ADD COLUMN creator_id integer REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE;

CREATE OR REPLACE FUNCTION post_aggregates_post ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        INSERT INTO post_aggregates (post_id, published, newest_comment_time, newest_comment_time_necro, category_id, creator_id)
            VALUES (NEW.id, NEW.published, NEW.published, NEW.published, NEW.category_id, NEW.creator_id);
    ELSIF (TG_OP = 'DELETE') THEN
        DELETE FROM post_aggregates
        WHERE post_id = OLD.id;
    END IF;
    RETURN NULL;
END
$$;

UPDATE
    post_aggregates
SET
    category_id = post.category_id,
    creator_id = post.creator_id
FROM
    post
WHERE
    post.id = post_aggregates.post_id;

ALTER TABLE post_aggregates
    ALTER COLUMN category_id SET NOT NULL,
    ALTER COLUMN creator_id SET NOT NULL;

