
-- generate new account with randomized name (max 20 chars) and set it

ALTER TABLE search_combined
    ADD CONSTRAINT search_combined_check CHECK (num_nonnulls (post_id, comment_id, community_id, person_id) = 1);

