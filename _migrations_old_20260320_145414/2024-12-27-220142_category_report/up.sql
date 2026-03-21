CREATE TABLE category_report (
    id serial PRIMARY KEY,
    creator_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    category_id int REFERENCES category ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    original_category_name text NOT NULL,
    original_category_title text NOT NULL,
    original_category_description text,
    original_category_sidebar text,
    original_category_icon text,
    original_category_banner text,
    reason text NOT NULL,
    resolved bool NOT NULL DEFAULT FALSE,
    resolver_id int REFERENCES person ON UPDATE CASCADE ON DELETE CASCADE,
    published timestamptz NOT NULL DEFAULT now(),
    updated timestamptz NULL,
    UNIQUE (category_id, creator_id)
);

CREATE INDEX idx_category_report_published ON category_report (published DESC);

ALTER TABLE report_combined
    ADD COLUMN category_report_id int UNIQUE REFERENCES category_report ON UPDATE CASCADE ON DELETE CASCADE,
    DROP CONSTRAINT report_combined_check,
    ADD CHECK (num_nonnulls (post_report_id, comment_report_id,  category_report_id) = 1);

ALTER TABLE category_aggregates
    ADD COLUMN report_count smallint NOT NULL DEFAULT 0,
    ADD COLUMN unresolved_report_count smallint NOT NULL DEFAULT 0;

