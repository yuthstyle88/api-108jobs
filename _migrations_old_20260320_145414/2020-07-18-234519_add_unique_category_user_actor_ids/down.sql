ALTER TABLE category
    ALTER COLUMN actor_id SET NOT NULL;

ALTER TABLE category
    ALTER COLUMN actor_id SET DEFAULT 'http://fake.com';

ALTER TABLE user_
    ALTER COLUMN actor_id SET NOT NULL;

ALTER TABLE user_
    ALTER COLUMN actor_id SET DEFAULT 'http://fake.com';

DROP FUNCTION generate_unique_changeme;

UPDATE
    category
SET
    actor_id = 'http://fake.com'
WHERE
    actor_id LIKE 'changeme_%';

UPDATE
    user_
SET
    actor_id = 'http://fake.com'
WHERE
    actor_id LIKE 'changeme_%';

DROP INDEX idx_user_lower_actor_id;

CREATE UNIQUE INDEX idx_user_name_lower_actor_id ON user_ (lower(name), lower(actor_id));

DROP INDEX idx_category_lower_actor_id;

