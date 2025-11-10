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

