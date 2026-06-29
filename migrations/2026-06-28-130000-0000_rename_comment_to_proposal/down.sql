-- Migration down: revert proposal → comment rename

-- 1. Drop CHECK constraints (they reference proposal column names)
ALTER TABLE modlog_combined DROP CONSTRAINT IF EXISTS modlog_combined_check;
ALTER TABLE inbox_combined DROP CONSTRAINT IF EXISTS inbox_combined_check;
ALTER TABLE report_combined DROP CONSTRAINT IF EXISTS report_combined_check;
ALTER TABLE search_combined DROP CONSTRAINT IF EXISTS search_combined_check;
ALTER TABLE person_content_combined DROP CONSTRAINT IF EXISTS person_content_combined_check;
ALTER TABLE person_saved_combined DROP CONSTRAINT IF EXISTS person_saved_combined_check;
ALTER TABLE person_liked_combined DROP CONSTRAINT IF EXISTS person_liked_combined_check;

-- 2. Rename leaf tables back
ALTER TABLE proposal_actions RENAME TO comment_actions;
ALTER TABLE proposal_reply RENAME TO comment_reply;
ALTER TABLE proposal_report RENAME TO comment_report;
ALTER TABLE admin_purge_proposal RENAME TO admin_purge_comment;
ALTER TABLE mod_remove_proposal RENAME TO mod_remove_comment;
ALTER TABLE person_proposal_mention RENAME TO person_comment_mention;

-- 3. Rename main table back
ALTER TABLE proposal RENAME TO comment;

-- 4. Rename FK columns back in combined tables
ALTER TABLE modlog_combined RENAME COLUMN admin_purge_proposal_id TO admin_purge_comment_id;
ALTER TABLE modlog_combined RENAME COLUMN mod_remove_proposal_id TO mod_remove_comment_id;
ALTER TABLE report_combined RENAME COLUMN proposal_report_id TO comment_report_id;
ALTER TABLE inbox_combined RENAME COLUMN proposal_reply_id TO comment_reply_id;
ALTER TABLE inbox_combined RENAME COLUMN person_proposal_mention_id TO person_comment_mention_id;
ALTER TABLE person_content_combined RENAME COLUMN proposal_id TO comment_id;
ALTER TABLE person_saved_combined RENAME COLUMN proposal_id TO comment_id;
ALTER TABLE person_liked_combined RENAME COLUMN proposal_id TO comment_id;
ALTER TABLE search_combined RENAME COLUMN proposal_id TO comment_id;

-- 5. Rename business table columns back
ALTER TABLE billing RENAME COLUMN proposal_id TO comment_id;
ALTER TABLE chat_room RENAME COLUMN current_proposal_id TO current_comment_id;
ALTER TABLE delivery_details RENAME COLUMN linked_proposal_id TO linked_comment_id;

-- 6. Rename person stat columns back
ALTER TABLE person RENAME COLUMN proposal_count TO comment_count;
ALTER TABLE person RENAME COLUMN proposal_score TO comment_score;

-- 7. Rename post counter columns back
ALTER TABLE post RENAME COLUMN proposals TO comments;
ALTER TABLE post RENAME COLUMN newest_proposal_time_necro_at TO newest_comment_time_necro_at;
ALTER TABLE post RENAME COLUMN newest_proposal_time_at TO newest_comment_time_at;

-- 8. Rename category/site stat columns back
ALTER TABLE category RENAME COLUMN proposals TO comments;
ALTER TABLE local_site RENAME COLUMN proposals TO comments;

-- 9. Rename rate limit columns back
ALTER TABLE local_site_rate_limit RENAME COLUMN proposal_max_requests TO comment_max_requests;
ALTER TABLE local_site_rate_limit RENAME COLUMN proposal_interval_seconds TO comment_interval_seconds;

-- 10. Rename post_actions columns back
ALTER TABLE post_actions RENAME COLUMN read_proposals_at TO read_comments_at;
ALTER TABLE post_actions RENAME COLUMN read_proposals_amount TO read_comments_amount;

-- 11. Rename local_user columns back
ALTER TABLE local_user RENAME COLUMN default_proposal_sort_type TO default_comment_sort_type;
ALTER TABLE local_user RENAME COLUMN collapse_bot_proposals TO collapse_bot_comments;

-- 12. Rename local_site column back
ALTER TABLE local_site RENAME COLUMN default_proposal_sort_type TO default_comment_sort_type;

-- 13. Rename indexes back
ALTER INDEX IF EXISTS idx_proposal_creator RENAME TO idx_comment_creator;
ALTER INDEX IF EXISTS idx_proposal_post RENAME TO idx_comment_post;
ALTER INDEX IF EXISTS idx_proposal_published RENAME TO idx_comment_published;
ALTER INDEX IF EXISTS idx_proposal_language RENAME TO idx_comment_language;
ALTER INDEX IF EXISTS idx_proposal_content_trigram RENAME TO idx_comment_content_trigram;
ALTER INDEX IF EXISTS idx_proposal_controversy RENAME TO idx_comment_controversy;
ALTER INDEX IF EXISTS idx_proposal_hot RENAME TO idx_comment_hot;
ALTER INDEX IF EXISTS idx_proposal_nonzero_hotrank RENAME TO idx_comment_nonzero_hotrank;
ALTER INDEX IF EXISTS idx_proposal_score RENAME TO idx_comment_score;
ALTER INDEX IF EXISTS idx_proposal_actions_liked_not_null RENAME TO idx_comment_actions_liked_not_null;
ALTER INDEX IF EXISTS idx_proposal_actions_saved_not_null RENAME TO idx_comment_actions_saved_not_null;
ALTER INDEX IF EXISTS idx_proposal_actions_like_score RENAME TO idx_comment_actions_like_score;
ALTER INDEX IF EXISTS idx_proposal_actions_proposal RENAME TO idx_comment_actions_comment;
ALTER INDEX IF EXISTS idx_proposal_reply_proposal RENAME TO idx_comment_reply_comment;
ALTER INDEX IF EXISTS idx_proposal_reply_recipient RENAME TO idx_comment_reply_recipient;
ALTER INDEX IF EXISTS idx_proposal_reply_published RENAME TO idx_comment_reply_published;
ALTER INDEX IF EXISTS idx_proposal_report_published RENAME TO idx_comment_report_published;
ALTER INDEX IF EXISTS idx_chat_room_current_proposal_id RENAME TO idx_chat_room_current_comment_id;
ALTER INDEX IF EXISTS idx_post_actions_read_proposals_not_null RENAME TO idx_post_actions_read_comments_not_null;
ALTER INDEX IF EXISTS idx_post_category_most_proposals RENAME TO idx_post_category_most_comments;
ALTER INDEX IF EXISTS idx_post_category_newest_proposal_time RENAME TO idx_post_category_newest_comment_time;
ALTER INDEX IF EXISTS idx_post_category_newest_proposal_time_necro RENAME TO idx_post_category_newest_comment_time_necro;
ALTER INDEX IF EXISTS idx_post_featured_category_most_proposals RENAME TO idx_post_featured_category_most_comments;
ALTER INDEX IF EXISTS idx_post_featured_category_newest_proposal_time RENAME TO idx_post_featured_category_newest_comment_time;
ALTER INDEX IF EXISTS idx_post_featured_category_newest_proposal_time_necr RENAME TO idx_post_featured_category_newest_comment_time_necr;
ALTER INDEX IF EXISTS idx_post_featured_local_most_proposals RENAME TO idx_post_featured_local_most_comments;
ALTER INDEX IF EXISTS idx_post_featured_local_newest_proposal_time RENAME TO idx_post_featured_local_newest_comment_time;
ALTER INDEX IF EXISTS idx_post_featured_local_newest_proposal_time_necro RENAME TO idx_post_featured_local_newest_comment_time_necro;
ALTER INDEX IF EXISTS idx_category_proposals RENAME TO idx_category_comments;

-- 13b. Rename constraint-backed indexes back
ALTER INDEX IF EXISTS admin_purge_proposal_pkey RENAME TO admin_purge_comment_pkey;
ALTER INDEX IF EXISTS mod_remove_proposal_pkey RENAME TO mod_remove_comment_pkey;
ALTER INDEX IF EXISTS proposal_pkey RENAME TO comment_pkey;
ALTER INDEX IF EXISTS proposal_actions_pkey RENAME TO comment_actions_pkey;
ALTER INDEX IF EXISTS proposal_reply_pkey RENAME TO comment_reply_pkey;
ALTER INDEX IF EXISTS proposal_reply_recipient_id_proposal_id_key RENAME TO comment_reply_recipient_id_comment_id_key;
ALTER INDEX IF EXISTS proposal_report_pkey RENAME TO comment_report_pkey;
ALTER INDEX IF EXISTS proposal_report_proposal_id_creator_id_key RENAME TO comment_report_comment_id_creator_id_key;
ALTER INDEX IF EXISTS inbox_combined_proposal_reply_id_key RENAME TO inbox_combined_comment_reply_id_key;
ALTER INDEX IF EXISTS inbox_combined_person_proposal_mention_id_key RENAME TO inbox_combined_person_comment_mention_id_key;
ALTER INDEX IF EXISTS modlog_combined_admin_purge_proposal_id_key RENAME TO modlog_combined_admin_purge_comment_id_key;
ALTER INDEX IF EXISTS modlog_combined_mod_remove_proposal_id_key RENAME TO modlog_combined_mod_remove_comment_id_key;
ALTER INDEX IF EXISTS person_content_combined_proposal_id_key RENAME TO person_content_combined_comment_id_key;
ALTER INDEX IF EXISTS person_liked_combined_person_id_proposal_id_key RENAME TO person_liked_combined_person_id_comment_id_key;
ALTER INDEX IF EXISTS person_mention_recipient_id_proposal_id_key RENAME TO person_mention_recipient_id_comment_id_key;
ALTER INDEX IF EXISTS person_saved_combined_person_id_proposal_id_key RENAME TO person_saved_combined_person_id_comment_id_key;
ALTER INDEX IF EXISTS report_combined_proposal_report_id_key RENAME TO report_combined_comment_report_id_key;
ALTER INDEX IF EXISTS search_combined_proposal_id_key RENAME TO search_combined_comment_id_key;

-- 14. Rename sequences back
ALTER SEQUENCE IF EXISTS proposal_id_seq RENAME TO comment_id_seq;
ALTER SEQUENCE IF EXISTS proposal_reply_id_seq RENAME TO comment_reply_id_seq;
ALTER SEQUENCE IF EXISTS proposal_report_id_seq RENAME TO comment_report_id_seq;
ALTER SEQUENCE IF EXISTS person_proposal_mention_id_seq RENAME TO person_comment_mention_id_seq;
ALTER SEQUENCE IF EXISTS admin_purge_proposal_id_seq RENAME TO admin_purge_comment_id_seq;
ALTER SEQUENCE IF EXISTS mod_remove_proposal_id_seq RENAME TO mod_remove_comment_id_seq;

-- 15. Rename enum type back
ALTER TYPE proposal_sort_type_enum RENAME TO comment_sort_type_enum;

-- 16. Re-add CHECK constraints with original comment column names
ALTER TABLE modlog_combined ADD CONSTRAINT modlog_combined_check CHECK (
  (num_nonnulls(admin_allow_instance_id, admin_block_instance_id, admin_purge_comment_id, admin_purge_category_id, admin_purge_person_id, admin_purge_post_id, mod_add_id, mod_add_category_id, mod_ban_id, mod_ban_from_category_id, mod_feature_post_id, mod_change_category_visibility_id, mod_lock_post_id, mod_remove_comment_id, mod_remove_category_id, mod_remove_post_id, mod_transfer_category_id) = 1)
);

ALTER TABLE inbox_combined ADD CONSTRAINT inbox_combined_check CHECK (
  (num_nonnulls(comment_reply_id, person_comment_mention_id, person_post_mention_id) = 1)
);

ALTER TABLE report_combined ADD CONSTRAINT report_combined_check CHECK (
  (num_nonnulls(post_report_id, comment_report_id, category_report_id) = 1)
);

ALTER TABLE search_combined ADD CONSTRAINT search_combined_check CHECK (
  (num_nonnulls(post_id, comment_id, category_id, person_id) = 1)
);

ALTER TABLE person_content_combined ADD CONSTRAINT person_content_combined_check CHECK (
  (num_nonnulls(post_id, comment_id) = 1)
);

ALTER TABLE person_saved_combined ADD CONSTRAINT person_saved_combined_check CHECK (
  (num_nonnulls(post_id, comment_id) = 1)
);

ALTER TABLE person_liked_combined ADD CONSTRAINT person_liked_combined_check CHECK (
  (num_nonnulls(post_id, comment_id) = 1)
);
