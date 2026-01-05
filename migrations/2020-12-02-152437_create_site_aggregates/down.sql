-- Site aggregates
DROP TABLE site_aggregates;

DROP TRIGGER site_aggregates_site ON site;

DROP TRIGGER site_aggregates_user_insert ON user_;

DROP TRIGGER site_aggregates_user_delete ON user_;

DROP TRIGGER site_aggregates_post_insert ON post;

DROP TRIGGER site_aggregates_post_delete ON post;

DROP TRIGGER site_aggregates_comment_insert ON comment;

DROP TRIGGER site_aggregates_comment_delete ON comment;

DROP TRIGGER site_aggregates_category_insert ON category;

DROP TRIGGER site_aggregates_category_delete ON category;

DROP FUNCTION site_aggregates_site, site_aggregates_user_insert, site_aggregates_user_delete, site_aggregates_post_insert, site_aggregates_post_delete, site_aggregates_comment_insert, site_aggregates_comment_delete, site_aggregates_category_insert, site_aggregates_category_delete;

