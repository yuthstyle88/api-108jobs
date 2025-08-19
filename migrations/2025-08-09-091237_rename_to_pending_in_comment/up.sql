ALTER TABLE comment
    RENAME COLUMN federation_pending TO pending;

ALTER TABLE comment
    ALTER COLUMN pending SET DEFAULT true;
