ALTER TABLE category
    ADD COLUMN followers_url varchar(255) NOT NULL DEFAULT generate_unique_changeme ();

ALTER TABLE category
    ADD COLUMN inbox_url varchar(255) NOT NULL DEFAULT generate_unique_changeme ();

ALTER TABLE category
    ADD COLUMN shared_inbox_url varchar(255);

ALTER TABLE user_
    ADD COLUMN inbox_url varchar(255) NOT NULL DEFAULT generate_unique_changeme ();

ALTER TABLE user_
    ADD COLUMN shared_inbox_url varchar(255);

ALTER TABLE category
    ADD CONSTRAINT idx_category_followers_url UNIQUE (followers_url);

ALTER TABLE category
    ADD CONSTRAINT idx_category_inbox_url UNIQUE (inbox_url);

ALTER TABLE user_
    ADD CONSTRAINT idx_user_inbox_url UNIQUE (inbox_url);

