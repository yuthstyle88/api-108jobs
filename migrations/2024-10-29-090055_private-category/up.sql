ALTER TYPE category_visibility
    ADD value 'Private';

-- Change `category_follower.pending` to `state` enum
CREATE TYPE category_follower_state AS enum (
    'Accepted',
    'Pending',
    'ApprovalRequired'
);

ALTER TABLE category_follower
    ALTER COLUMN pending DROP DEFAULT;

CREATE OR REPLACE FUNCTION convert_follower_state (b bool)
    RETURNS category_follower_state
    LANGUAGE sql
    IMMUTABLE PARALLEL SAFE
    AS $$
    SELECT
        CASE WHEN b = TRUE THEN
            'Pending'::category_follower_state
        ELSE
            'Accepted'::category_follower_state
        END
$$;

ALTER TABLE category_follower
    ALTER COLUMN pending TYPE category_follower_state
    USING convert_follower_state (pending);

DROP FUNCTION convert_follower_state;

ALTER TABLE category_follower RENAME COLUMN pending TO state;

-- Add column for mod who approved the private category follower
-- Dont use foreign key here, otherwise joining to person table doesnt work easily
ALTER TABLE category_follower
    ADD COLUMN approver_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE;

-- Enable signed fetch, necessary to fetch content in private communities


