ALTER TABLE category
    DROP COLUMN description;

ALTER TABLE category RENAME COLUMN sidebar TO description;

