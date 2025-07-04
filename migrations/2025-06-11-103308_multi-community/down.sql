ALTER TABLE search_combined
    DROP CONSTRAINT search_combined_check;

ALTER TABLE search_combined
    ADD CONSTRAINT search_combined_check CHECK (num_nonnulls (post_id, comment_id, community_id, person_id) = 1);


CREATE TYPE listing_type_enum_tmp AS ENUM (
    'All',
    'Local',
    'Subscribed',
    'ModeratorView'
);


ALTER TABLE local_user
    ALTER COLUMN default_listing_type DROP DEFAULT,
    ALTER COLUMN default_listing_type TYPE listing_type_enum_tmp
    USING (default_listing_type::text::listing_type_enum_tmp),
    ALTER COLUMN default_listing_type SET DEFAULT 'Local';

ALTER TABLE local_site
    ALTER COLUMN default_post_listing_type DROP DEFAULT,
    ALTER COLUMN default_post_listing_type TYPE listing_type_enum_tmp
    USING (default_post_listing_type::text::listing_type_enum_tmp),
    ALTER COLUMN default_post_listing_type SET DEFAULT 'Local';

DROP TYPE listing_type_enum;

ALTER TYPE listing_type_enum_tmp RENAME TO listing_type_enum;

