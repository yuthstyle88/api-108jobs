DROP TABLE activity;

ALTER TABLE user_
    DROP COLUMN actor_id,
    DROP COLUMN bio,
    DROP COLUMN local,
    DROP COLUMN last_refreshed_at;

ALTER TABLE category
    DROP COLUMN actor_id,
    DROP COLUMN local,
    DROP COLUMN last_refreshed_at;

