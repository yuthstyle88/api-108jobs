-- Revert timestamp column renames (idempotent version)
DO $$
BEGIN
  -- Helper function to rename column if exists
  PERFORM 1;

  -- admin_allow_instance
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'admin_allow_instance' AND column_name = 'published_at') THEN
    ALTER TABLE admin_allow_instance RENAME COLUMN published_at TO published;
  END IF;

  -- admin_block_instance
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'admin_block_instance' AND column_name = 'expires_at') THEN
    ALTER TABLE admin_block_instance RENAME COLUMN expires_at TO expires;
  END IF;
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'admin_block_instance' AND column_name = 'published_at') THEN
    ALTER TABLE admin_block_instance RENAME COLUMN published_at TO published;
  END IF;

  -- admin_purge_comment
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'admin_purge_comment' AND column_name = 'published_at') THEN
    ALTER TABLE admin_purge_comment RENAME COLUMN published_at TO published;
  END IF;

  -- admin_purge_category
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'admin_purge_category' AND column_name = 'published_at') THEN
    ALTER TABLE admin_purge_category RENAME COLUMN published_at TO published;
  END IF;

  -- admin_purge_person
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'admin_purge_person' AND column_name = 'published_at') THEN
    ALTER TABLE admin_purge_person RENAME COLUMN published_at TO published;
  END IF;

  -- admin_purge_post
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'admin_purge_post' AND column_name = 'published_at') THEN
    ALTER TABLE admin_purge_post RENAME COLUMN published_at TO published;
  END IF;

  -- captcha_answer
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'captcha_answer' AND column_name = 'published_at') THEN
    ALTER TABLE captcha_answer RENAME COLUMN published_at TO published;
  END IF;

  -- comment
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'comment' AND column_name = 'published_at') THEN
    ALTER TABLE comment RENAME COLUMN published_at TO published;
  END IF;
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'comment' AND column_name = 'updated_at') THEN
    ALTER TABLE comment RENAME COLUMN updated_at TO updated;
  END IF;

  -- comment_actions
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'comment_actions' AND column_name = 'liked_at') THEN
    ALTER TABLE comment_actions RENAME COLUMN liked_at TO liked;
  END IF;
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'comment_actions' AND column_name = 'saved_at') THEN
    ALTER TABLE comment_actions RENAME COLUMN saved_at TO saved;
  END IF;

  -- comment_reply
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'comment_reply' AND column_name = 'published_at') THEN
    ALTER TABLE comment_reply RENAME COLUMN published_at TO published;
  END IF;

  -- comment_report
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'comment_report' AND column_name = 'published_at') THEN
    ALTER TABLE comment_report RENAME COLUMN published_at TO published;
  END IF;
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'comment_report' AND column_name = 'updated_at') THEN
    ALTER TABLE comment_report RENAME COLUMN updated_at TO updated;
  END IF;

  -- category
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'category' AND column_name = 'published_at') THEN
    ALTER TABLE category RENAME COLUMN published_at TO published;
  END IF;
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'category' AND column_name = 'updated_at') THEN
    ALTER TABLE category RENAME COLUMN updated_at TO updated;
  END IF;

  -- category_actions
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'category_actions' AND column_name = 'followed_at') THEN
    ALTER TABLE category_actions RENAME COLUMN followed_at TO followed;
  END IF;
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'category_actions' AND column_name = 'blocked_at') THEN
    ALTER TABLE category_actions RENAME COLUMN blocked_at TO blocked;
  END IF;
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'category_actions' AND column_name = 'became_moderator_at') THEN
    ALTER TABLE category_actions RENAME COLUMN became_moderator_at TO became_moderator;
  END IF;
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'category_actions' AND column_name = 'received_ban_at') THEN
    ALTER TABLE category_actions RENAME COLUMN received_ban_at TO received_ban;
  END IF;
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'category_actions' AND column_name = 'ban_expires_at') THEN
    ALTER TABLE category_actions RENAME COLUMN ban_expires_at TO ban_expires;
  END IF;

  -- category_report
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'category_report' AND column_name = 'published_at') THEN
    ALTER TABLE category_report RENAME COLUMN published_at TO published;
  END IF;
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'category_report' AND column_name = 'updated_at') THEN
    ALTER TABLE category_report RENAME COLUMN updated_at TO updated;
  END IF;

  -- custom_emoji
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'custom_emoji' AND column_name = 'published_at') THEN
    ALTER TABLE custom_emoji RENAME COLUMN published_at TO published;
  END IF;
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'custom_emoji' AND column_name = 'updated_at') THEN
    ALTER TABLE custom_emoji RENAME COLUMN updated_at TO updated;
  END IF;

  -- email_verification
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'email_verification' AND column_name = 'published_at') THEN
    ALTER TABLE email_verification RENAME COLUMN published_at TO published;
  END IF;

  -- inbox_combined
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'inbox_combined' AND column_name = 'published_at') THEN
    ALTER TABLE inbox_combined RENAME COLUMN published_at TO published;
  END IF;

  -- instance
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'instance' AND column_name = 'published_at') THEN
    ALTER TABLE instance RENAME COLUMN published_at TO published;
  END IF;
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'instance' AND column_name = 'updated_at') THEN
    ALTER TABLE instance RENAME COLUMN updated_at TO updated;
  END IF;

  -- instance_actions
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'instance_actions' AND column_name = 'blocked_at') THEN
    ALTER TABLE instance_actions RENAME COLUMN blocked_at TO blocked;
  END IF;
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'instance_actions' AND column_name = 'received_ban_at') THEN
    ALTER TABLE instance_actions RENAME COLUMN received_ban_at TO received_ban;
  END IF;
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'instance_actions' AND column_name = 'ban_expires_at') THEN
    ALTER TABLE instance_actions RENAME COLUMN ban_expires_at TO ban_expires;
  END IF;

  -- local_image
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'local_image' AND column_name = 'published_at') THEN
    ALTER TABLE local_image RENAME COLUMN published_at TO published;
  END IF;

  -- local_site
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'local_site' AND column_name = 'published_at') THEN
    ALTER TABLE local_site RENAME COLUMN published_at TO published;
  END IF;
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'local_site' AND column_name = 'updated_at') THEN
    ALTER TABLE local_site RENAME COLUMN updated_at TO updated;
  END IF;

  -- local_site_rate_limit
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'local_site_rate_limit' AND column_name = 'published_at') THEN
    ALTER TABLE local_site_rate_limit RENAME COLUMN published_at TO published;
  END IF;
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'local_site_rate_limit' AND column_name = 'updated_at') THEN
    ALTER TABLE local_site_rate_limit RENAME COLUMN updated_at TO updated;
  END IF;

  -- local_site_url_blocklist
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'local_site_url_blocklist' AND column_name = 'published_at') THEN
    ALTER TABLE local_site_url_blocklist RENAME COLUMN published_at TO published;
  END IF;
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'local_site_url_blocklist' AND column_name = 'updated_at') THEN
    ALTER TABLE local_site_url_blocklist RENAME COLUMN updated_at TO updated;
  END IF;

  -- local_user
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'local_user' AND column_name = 'last_donation_notification_at') THEN
    ALTER TABLE local_user RENAME COLUMN last_donation_notification_at TO last_donation_notification;
  END IF;

  -- login_token
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'login_token' AND column_name = 'published_at') THEN
    ALTER TABLE login_token RENAME COLUMN published_at TO published;
  END IF;

  -- mod_add
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'mod_add' AND column_name = 'published_at') THEN
    ALTER TABLE mod_add RENAME COLUMN published_at TO published;
  END IF;

  -- mod_add_category
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'mod_add_category' AND column_name = 'published_at') THEN
    ALTER TABLE mod_add_category RENAME COLUMN published_at TO published;
  END IF;

  -- mod_ban
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'mod_ban' AND column_name = 'published_at') THEN
    ALTER TABLE mod_ban RENAME COLUMN published_at TO published;
  END IF;
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'mod_ban' AND column_name = 'expires_at') THEN
    ALTER TABLE mod_ban RENAME COLUMN expires_at TO expires;
  END IF;

  -- mod_ban_from_category
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'mod_ban_from_category' AND column_name = 'published_at') THEN
    ALTER TABLE mod_ban_from_category RENAME COLUMN published_at TO published;
  END IF;
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'mod_ban_from_category' AND column_name = 'expires_at') THEN
    ALTER TABLE mod_ban_from_category RENAME COLUMN expires_at TO expires;
  END IF;

  -- mod_change_category_visibility
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'mod_change_category_visibility' AND column_name = 'published_at') THEN
    ALTER TABLE mod_change_category_visibility RENAME COLUMN published_at TO published;
  END IF;

  -- mod_feature_post
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'mod_feature_post' AND column_name = 'published_at') THEN
    ALTER TABLE mod_feature_post RENAME COLUMN published_at TO published;
  END IF;

  -- mod_lock_post
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'mod_lock_post' AND column_name = 'published_at') THEN
    ALTER TABLE mod_lock_post RENAME COLUMN published_at TO published;
  END IF;

  -- mod_remove_comment
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'mod_remove_comment' AND column_name = 'published_at') THEN
    ALTER TABLE mod_remove_comment RENAME COLUMN published_at TO published;
  END IF;

  -- mod_remove_category
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'mod_remove_category' AND column_name = 'published_at') THEN
    ALTER TABLE mod_remove_category RENAME COLUMN published_at TO published;
  END IF;

  -- mod_remove_post
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'mod_remove_post' AND column_name = 'published_at') THEN
    ALTER TABLE mod_remove_post RENAME COLUMN published_at TO published;
  END IF;

  -- mod_transfer_category
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'mod_transfer_category' AND column_name = 'published_at') THEN
    ALTER TABLE mod_transfer_category RENAME COLUMN published_at TO published;
  END IF;

  -- modlog_combined
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'modlog_combined' AND column_name = 'published_at') THEN
    ALTER TABLE modlog_combined RENAME COLUMN published_at TO published;
  END IF;

  -- oauth_account
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'oauth_account' AND column_name = 'published_at') THEN
    ALTER TABLE oauth_account RENAME COLUMN published_at TO published;
  END IF;
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'oauth_account' AND column_name = 'updated_at') THEN
    ALTER TABLE oauth_account RENAME COLUMN updated_at TO updated;
  END IF;

  -- password_reset_request
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'password_reset_request' AND column_name = 'published_at') THEN
    ALTER TABLE password_reset_request RENAME COLUMN published_at TO published;
  END IF;

  -- person
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'person' AND column_name = 'published_at') THEN
    ALTER TABLE person RENAME COLUMN published_at TO published;
  END IF;
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'person' AND column_name = 'updated_at') THEN
    ALTER TABLE person RENAME COLUMN updated_at TO updated;
  END IF;

  -- person_actions
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'person_actions' AND column_name = 'followed_at') THEN
    ALTER TABLE person_actions RENAME COLUMN followed_at TO followed;
  END IF;
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'person_actions' AND column_name = 'blocked_at') THEN
    ALTER TABLE person_actions RENAME COLUMN blocked_at TO blocked;
  END IF;

  -- person_ban
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'person_ban' AND column_name = 'published_at') THEN
    ALTER TABLE person_ban RENAME COLUMN published_at TO published;
  END IF;

  -- person_comment_mention
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'person_comment_mention' AND column_name = 'published_at') THEN
    ALTER TABLE person_comment_mention RENAME COLUMN published_at TO published;
  END IF;

  -- person_content_combined
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'person_content_combined' AND column_name = 'published_at') THEN
    ALTER TABLE person_content_combined RENAME COLUMN published_at TO published;
  END IF;

  -- person_liked_combined
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'person_liked_combined' AND column_name = 'liked_at') THEN
    ALTER TABLE person_liked_combined RENAME COLUMN liked_at TO liked;
  END IF;

  -- person_post_mention
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'person_post_mention' AND column_name = 'published_at') THEN
    ALTER TABLE person_post_mention RENAME COLUMN published_at TO published;
  END IF;

  -- person_saved_combined
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'person_saved_combined' AND column_name = 'saved_at') THEN
    ALTER TABLE person_saved_combined RENAME COLUMN saved_at TO saved;
  END IF;

  -- post
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'post' AND column_name = 'published_at') THEN
    ALTER TABLE post RENAME COLUMN published_at TO published;
  END IF;
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'post' AND column_name = 'updated_at') THEN
    ALTER TABLE post RENAME COLUMN updated_at TO updated;
  END IF;
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'post' AND column_name = 'scheduled_publish_time_at') THEN
    ALTER TABLE post RENAME COLUMN scheduled_publish_time_at TO scheduled_publish_time;
  END IF;
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'post' AND column_name = 'newest_comment_time_at') THEN
    ALTER TABLE post RENAME COLUMN newest_comment_time_at TO newest_comment_time;
  END IF;
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'post' AND column_name = 'newest_comment_time_necro_at') THEN
    ALTER TABLE post RENAME COLUMN newest_comment_time_necro_at TO newest_comment_time_necro;
  END IF;

  -- post_actions
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'post_actions' AND column_name = 'read_at') THEN
    ALTER TABLE post_actions RENAME COLUMN read_at TO read;
  END IF;
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'post_actions' AND column_name = 'read_comments_at') THEN
    ALTER TABLE post_actions RENAME COLUMN read_comments_at TO read_comments;
  END IF;
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'post_actions' AND column_name = 'saved_at') THEN
    ALTER TABLE post_actions RENAME COLUMN saved_at TO saved;
  END IF;
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'post_actions' AND column_name = 'liked_at') THEN
    ALTER TABLE post_actions RENAME COLUMN liked_at TO liked;
  END IF;
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'post_actions' AND column_name = 'hidden_at') THEN
    ALTER TABLE post_actions RENAME COLUMN hidden_at TO hidden;
  END IF;

  -- post_report
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'post_report' AND column_name = 'published_at') THEN
    ALTER TABLE post_report RENAME COLUMN published_at TO published;
  END IF;
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'post_report' AND column_name = 'updated_at') THEN
    ALTER TABLE post_report RENAME COLUMN updated_at TO updated;
  END IF;

  -- post_tag
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'post_tag' AND column_name = 'published_at') THEN
    ALTER TABLE post_tag RENAME COLUMN published_at TO published;
  END IF;

  -- received_activity
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'received_activity' AND column_name = 'published_at') THEN
    ALTER TABLE received_activity RENAME COLUMN published_at TO published;
  END IF;

  -- registration_application
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'registration_application' AND column_name = 'published_at') THEN
    ALTER TABLE registration_application RENAME COLUMN published_at TO published;
  END IF;

  -- remote_image
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'remote_image' AND column_name = 'published_at') THEN
    ALTER TABLE remote_image RENAME COLUMN published_at TO published;
  END IF;

  -- report_combined
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'report_combined' AND column_name = 'published_at') THEN
    ALTER TABLE report_combined RENAME COLUMN published_at TO published;
  END IF;

  -- search_combined
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'search_combined' AND column_name = 'published_at') THEN
    ALTER TABLE search_combined RENAME COLUMN published_at TO published;
  END IF;

  -- sent_activity
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'sent_activity' AND column_name = 'published_at') THEN
    ALTER TABLE sent_activity RENAME COLUMN published_at TO published;
  END IF;

  -- site
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'site' AND column_name = 'published_at') THEN
    ALTER TABLE site RENAME COLUMN published_at TO published;
  END IF;
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'site' AND column_name = 'updated_at') THEN
    ALTER TABLE site RENAME COLUMN updated_at TO updated;
  END IF;

  -- tag
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'tag' AND column_name = 'published_at') THEN
    ALTER TABLE tag RENAME COLUMN published_at TO published;
  END IF;
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'tag' AND column_name = 'updated_at') THEN
    ALTER TABLE tag RENAME COLUMN updated_at TO updated;
  END IF;

  -- tagline
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'tagline' AND column_name = 'published_at') THEN
    ALTER TABLE tagline RENAME COLUMN published_at TO published;
  END IF;
  IF EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'tagline' AND column_name = 'updated_at') THEN
    ALTER TABLE tagline RENAME COLUMN updated_at TO updated;
  END IF;

END $$;
