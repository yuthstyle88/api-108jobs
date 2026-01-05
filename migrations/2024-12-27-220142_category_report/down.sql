DELETE FROM report_combined
WHERE category_report_id IS NOT NULL;

ALTER TABLE report_combined
    DROP CONSTRAINT report_combined_check,
    ADD CHECK (num_nonnulls (post_report_id, comment_report_id) = 1),
    DROP COLUMN category_report_id;

DROP TABLE category_report CASCADE;

ALTER TABLE category_aggregates
    DROP COLUMN report_count,
    DROP COLUMN unresolved_report_count;

