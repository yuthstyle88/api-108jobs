-- Couldn't find a way to put subscribers_local right after subscribers except recreating the table.
ALTER TABLE category_aggregates
    ADD COLUMN subscribers_local bigint NOT NULL DEFAULT 0;

-- update initial value
-- update by counting local persons who follow communities.
WITH follower_counts AS (
    SELECT
        category_id,
        count(*) AS local_sub_count
    FROM
        category_follower cf
        JOIN person p ON p.id = cf.person_id
    WHERE
        p.local = TRUE
    GROUP BY
        category_id)
UPDATE
    category_aggregates ca
SET
    subscribers_local = local_sub_count
FROM
    follower_counts
WHERE
    ca.category_id = follower_counts.category_id;

-- subscribers should be updated only when a local category is followed by a local or remote person
-- subscribers_local should be updated only when a local person follows a local or remote category
CREATE OR REPLACE FUNCTION category_aggregates_subscriber_count ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        UPDATE
            category_aggregates ca
        SET
            subscribers = subscribers + category.local::int,
            subscribers_local = subscribers_local + person.local::int
        FROM
            category
        LEFT JOIN person ON person.id = NEW.person_id
    WHERE
        category.id = NEW.category_id
            AND category.id = ca.category_id
            AND person.local IS NOT NULL;
    ELSIF (TG_OP = 'DELETE') THEN
        UPDATE
            category_aggregates ca
        SET
            subscribers = subscribers - category.local::int,
            subscribers_local = subscribers_local - person.local::int
        FROM
            category
        LEFT JOIN person ON person.id = OLD.person_id
    WHERE
        category.id = OLD.category_id
            AND category.id = ca.category_id
            AND person.local IS NOT NULL;
    END IF;
    RETURN NULL;
END
$$;

-- to be able to join person on the trigger above, we need to run it before the person is deleted: https://github.com/LemmyNet/lemmy/pull/4166#issuecomment-1874095856
CREATE FUNCTION delete_follow_before_person ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    DELETE FROM category_follower AS c
    WHERE c.person_id = OLD.id;
    RETURN OLD;
END;
$$;

CREATE TRIGGER delete_follow_before_person
    BEFORE DELETE ON person
    FOR EACH ROW
    EXECUTE FUNCTION delete_follow_before_person ();

