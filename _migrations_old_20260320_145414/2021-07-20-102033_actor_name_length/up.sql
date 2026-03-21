DROP VIEW person_alias_1;

DROP VIEW person_alias_2;

ALTER TABLE category
    ALTER COLUMN name TYPE varchar(255);

ALTER TABLE category
    ALTER COLUMN title TYPE varchar(255);

ALTER TABLE person
    ALTER COLUMN name TYPE varchar(255);

ALTER TABLE person
    ALTER COLUMN display_name TYPE varchar(255);

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

