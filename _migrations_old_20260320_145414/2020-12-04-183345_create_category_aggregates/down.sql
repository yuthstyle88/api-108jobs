-- category aggregates
DROP TABLE category_aggregates;

DROP TRIGGER category_aggregates_category ON category;

DROP TRIGGER category_aggregates_post_count ON post;

DROP TRIGGER category_aggregates_comment_count ON comment;

DROP TRIGGER category_aggregates_subscriber_count ON category_follower;

DROP FUNCTION category_aggregates_category, category_aggregates_post_count, category_aggregates_comment_count, category_aggregates_subscriber_count;

