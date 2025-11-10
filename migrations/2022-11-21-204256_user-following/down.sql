DROP TABLE person_follower;

ALTER TABLE category_follower
    ALTER COLUMN pending DROP NOT NULL;

