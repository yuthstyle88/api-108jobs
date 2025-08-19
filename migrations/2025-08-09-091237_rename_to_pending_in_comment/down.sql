ALTER TABLE comment
    RENAME COLUMN pending TO federation_pending;

ALTER TABLE comment
    ALTER COLUMN federation_pending DROP DEFAULT;
