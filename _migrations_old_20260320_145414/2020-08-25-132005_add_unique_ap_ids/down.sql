-- Drop the uniques
ALTER TABLE post
    DROP CONSTRAINT idx_post_ap_id;

ALTER TABLE comment
    DROP CONSTRAINT idx_comment_ap_id;

ALTER TABLE user_
    DROP CONSTRAINT idx_user_actor_id;

ALTER TABLE category
    DROP CONSTRAINT idx_category_actor_id;


ALTER TABLE post
    ALTER COLUMN ap_id SET NOT NULL;

ALTER TABLE post
    ALTER COLUMN ap_id SET DEFAULT 'http://fake.com';

ALTER TABLE comment
    ALTER COLUMN ap_id SET NOT NULL;

ALTER TABLE comment
    ALTER COLUMN ap_id SET DEFAULT 'http://fake.com';


UPDATE
    post
SET
    ap_id = 'http://fake.com'
WHERE
    ap_id LIKE 'changeme_%';

UPDATE
    comment
SET
    ap_id = 'http://fake.com'
WHERE
    ap_id LIKE 'changeme_%';

