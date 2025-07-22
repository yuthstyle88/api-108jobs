ALTER TABLE community
    ADD COLUMN path ltree,
ADD COLUMN subtitle   text,
ADD COLUMN slug       text         NOT NULL,
ADD COLUMN active     boolean      NOT NULL DEFAULT true,
ADD COLUMN is_new     boolean;
