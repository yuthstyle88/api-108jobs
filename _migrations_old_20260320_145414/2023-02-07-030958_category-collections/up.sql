ALTER TABLE category
    ADD COLUMN moderators_url varchar(255) UNIQUE;

ALTER TABLE category
    ADD COLUMN featured_url varchar(255) UNIQUE;

