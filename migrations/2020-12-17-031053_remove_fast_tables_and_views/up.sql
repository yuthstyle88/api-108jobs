-- Drop triggers
DROP TRIGGER IF EXISTS refresh_comment ON comment;

DROP TRIGGER IF EXISTS refresh_comment_like ON comment_like;

DROP TRIGGER IF EXISTS refresh_category ON category;

DROP TRIGGER IF EXISTS refresh_category_follower ON category_follower;

DROP TRIGGER IF EXISTS refresh_category_user_ban ON category_user_ban;

DROP TRIGGER IF EXISTS refresh_post ON post;

DROP TRIGGER IF EXISTS refresh_post_like ON post_like;

DROP TRIGGER IF EXISTS refresh_user ON user_;

-- Drop functions
DROP FUNCTION IF EXISTS refresh_comment, refresh_comment_like, refresh_category, refresh_category_follower, refresh_category_user_ban, refresh_post, refresh_post_like, refresh_user CASCADE;

-- Drop views
DROP VIEW IF EXISTS comment_aggregates_view, comment_fast_view, comment_report_view, comment_view, category_aggregates_view, category_fast_view, category_follower_view, category_moderator_view, category_user_ban_view, category_view, mod_add_category_view, mod_add_view, mod_ban_from_category_view, mod_ban_view, mod_lock_post_view, mod_remove_comment_view, mod_remove_category_view, mod_remove_post_view, mod_sticky_post_view, post_aggregates_view, post_fast_view, post_report_view, post_view, reply_fast_view, site_view, user_mention_fast_view, user_mention_view, user_view CASCADE;

-- Drop fast tables
DROP TABLE IF EXISTS comment_aggregates_fast, category_aggregates_fast, post_aggregates_fast, user_fast CASCADE;

