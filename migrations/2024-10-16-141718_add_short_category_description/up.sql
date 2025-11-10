-- Renaming description to sidebar
ALTER TABLE category RENAME COLUMN description TO sidebar;

-- Adding a short description column
ALTER TABLE category
    ADD COLUMN description varchar(150);

