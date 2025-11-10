CREATE TABLE instance_block (
    id serial PRIMARY KEY,
    person_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    instance_id int REFERENCES instance ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    published timestamptz NOT NULL DEFAULT now(),
    UNIQUE (person_id, instance_id)
);

ALTER TABLE post_aggregates
    ADD COLUMN instance_id integer REFERENCES instance ON UPDATE CASCADE ON DELETE CASCADE;

CREATE OR REPLACE FUNCTION post_aggregates_post ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        INSERT INTO post_aggregates (post_id, published, newest_comment_time, newest_comment_time_necro, category_id, creator_id, instance_id)
        SELECT
            NEW.id,
            NEW.published,
            NEW.published,
            NEW.published,
            NEW.category_id,
            NEW.creator_id,
            category.instance_id
        FROM
            category
        WHERE
            NEW.category_id = category.id;
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
    instance_id = category.instance_id
FROM
    post
    JOIN category ON post.category_id = category.id
WHERE
    post.id = post_aggregates.post_id;

ALTER TABLE post_aggregates
    ALTER COLUMN instance_id SET NOT NULL;

