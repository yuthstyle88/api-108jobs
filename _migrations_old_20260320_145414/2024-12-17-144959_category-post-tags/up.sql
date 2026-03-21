-- a tag is a federatable object that gives additional context to another object, which can be displayed and filtered on
-- currently, we only have category post tags, which is a tag that is created by post authors as well as mods  of a category,
-- to categorize a post. in the future we may add more tag types, depending on the requirements,
-- this will lead to either expansion of this table (category_id optional, addition of tag_type enum)
-- or split of this table / creation of new tables.
CREATE TABLE tag (
    id serial PRIMARY KEY,
    ap_id text NOT NULL UNIQUE,
    display_name text NOT NULL,
    category_id int NOT NULL REFERENCES category (id) ON UPDATE CASCADE ON DELETE CASCADE,
    published timestamptz NOT NULL DEFAULT now(),
    updated timestamptz,
    deleted boolean NOT NULL DEFAULT FALSE
);

-- an association between a post and a tag. created/updated by the post author or mods of a category
CREATE TABLE post_tag (
    post_id int NOT NULL REFERENCES post (id) ON UPDATE CASCADE ON DELETE CASCADE,
    tag_id int NOT NULL REFERENCES tag (id) ON UPDATE CASCADE ON DELETE CASCADE,
    published timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (post_id, tag_id)
);

