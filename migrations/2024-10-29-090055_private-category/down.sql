-- Remove private visibility
ALTER TYPE category_visibility RENAME TO category_visibility__;

CREATE TYPE category_visibility AS enum (
    'Public',
    'LocalOnly'
);

ALTER TABLE category
    ALTER COLUMN visibility DROP DEFAULT;

ALTER TABLE category
    ALTER COLUMN visibility TYPE category_visibility
    USING visibility::text::category_visibility;

ALTER TABLE category
    ALTER COLUMN visibility SET DEFAULT 'Public';

DROP TYPE category_visibility__;

-- Revert category follower changes
CREATE OR REPLACE FUNCTION convert_follower_state (s category_follower_state)
    RETURNS bool
    LANGUAGE sql
    AS $$
    SELECT
        CASE WHEN s = 'Pending' THEN
            TRUE
        ELSE
            FALSE
        END
$$;

ALTER TABLE category_follower
    ALTER COLUMN state TYPE bool
    USING convert_follower_state (state);

DROP FUNCTION convert_follower_state;

ALTER TABLE category_follower
    ALTER COLUMN state SET DEFAULT FALSE;

ALTER TABLE category_follower RENAME COLUMN state TO pending;

DROP TYPE category_follower_state;

ALTER TABLE category_follower
    DROP COLUMN approver_id;


