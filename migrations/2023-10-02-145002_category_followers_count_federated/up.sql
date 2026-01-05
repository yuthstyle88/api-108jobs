-- The subscriber count should only be updated for local communities. For remote
-- communities it is read over federation from the origin instance.
CREATE OR REPLACE FUNCTION category_aggregates_subscriber_count ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        UPDATE
            category_aggregates
        SET
            subscribers = subscribers + 1
        FROM
            category
        WHERE
            category.id = category_id
            AND category.local
            AND category_id = NEW.category_id;
    ELSIF (TG_OP = 'DELETE') THEN
        UPDATE
            category_aggregates
        SET
            subscribers = subscribers - 1
        FROM
            category
        WHERE
            category.id = category_id
            AND category.local
            AND category_id = OLD.category_id;
    END IF;
    RETURN NULL;
END
$$;

