DROP VIEW person_alias_1;

DROP VIEW person_alias_2;

-- Pre-truncate potentially long values to fit the legacy limits to avoid
-- "value too long for type character varying(20)" during ALTERs below.
UPDATE category
SET name = LEFT(name, 20)
WHERE char_length(name) > 20;

UPDATE person
SET name = LEFT(name, 20)
WHERE char_length(name) > 20;

UPDATE person
SET display_name = LEFT(display_name, 20)
WHERE display_name IS NOT NULL AND char_length(display_name) > 20;

ALTER TABLE category
    ALTER COLUMN name TYPE varchar(20);

ALTER TABLE category
    ALTER COLUMN title TYPE varchar(100);

ALTER TABLE person
    ALTER COLUMN name TYPE varchar(20);

ALTER TABLE person
    ALTER COLUMN display_name TYPE varchar(20);

CREATE VIEW person_alias_1 AS
SELECT
    *
FROM
    person;

CREATE VIEW person_alias_2 AS
SELECT
    *
FROM
    person;

