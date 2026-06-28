-- Migration: rename comment → proposal
-- Generated: 2026-06-28

-- 1. Drop CHECK constraints on combined tables (they reference comment column names)
ALTER TABLE modlog_combined DROP CONSTRAINT IF EXISTS modlog_combined_check;
ALTER TABLE inbox_combined DROP CONSTRAINT IF EXISTS inbox_combined_check;
ALTER TABLE report_combined DROP CONSTRAINT IF EXISTS report_combined_check;
ALTER TABLE search_combined DROP CONSTRAINT IF EXISTS search_combined_check;
ALTER TABLE person_content_combined DROP CONSTRAINT IF EXISTS person_content_combined_check;
ALTER TABLE person_saved_combined DROP CONSTRAINT IF EXISTS person_saved_combined_check;
ALTER TABLE person_liked_combined DROP CONSTRAINT IF EXISTS person_liked_combined_check;

-- 2. Rename leaf tables first (no tables FK into them)
ALTER TABLE comment_actions RENAME TO proposal_actions;
ALTER TABLE comment_reply RENAME TO proposal_reply;
ALTER TABLE comment_report RENAME TO proposal_report;
ALTER TABLE admin_purge_comment RENAME TO admin_purge_proposal;
ALTER TABLE mod_remove_comment RENAME TO mod_remove_proposal;
ALTER TABLE person_comment_mention RENAME TO person_proposal_mention;

-- 3. Rename main table last
ALTER TABLE comment RENAME TO proposal;

-- 4. Rename FK columns in combined tables
ALTER TABLE modlog_combined RENAME COLUMN admin_purge_comment_id TO admin_purge_proposal_id;
ALTER TABLE modlog_combined RENAME COLUMN mod_remove_comment_id TO mod_remove_proposal_id;
ALTER TABLE report_combined RENAME COLUMN comment_report_id TO proposal_report_id;
ALTER TABLE inbox_combined RENAME COLUMN comment_reply_id TO proposal_reply_id;
ALTER TABLE inbox_combined RENAME COLUMN person_comment_mention_id TO person_proposal_mention_id;
ALTER TABLE person_content_combined RENAME COLUMN comment_id TO proposal_id;
ALTER TABLE person_saved_combined RENAME COLUMN comment_id TO proposal_id;
ALTER TABLE person_liked_combined RENAME COLUMN comment_id TO proposal_id;
ALTER TABLE search_combined RENAME COLUMN comment_id TO proposal_id;

-- 5. Rename business table columns
ALTER TABLE billing RENAME COLUMN comment_id TO proposal_id;
ALTER TABLE chat_room RENAME COLUMN current_comment_id TO current_proposal_id;
ALTER TABLE delivery_details RENAME COLUMN linked_comment_id TO linked_proposal_id;

-- 6. Rename person stat columns
ALTER TABLE person RENAME COLUMN comment_count TO proposal_count;
ALTER TABLE person RENAME COLUMN comment_score TO proposal_score;

-- 7. Rename post counter columns
ALTER TABLE post RENAME COLUMN comments TO proposals;
ALTER TABLE post RENAME COLUMN newest_comment_time_necro_at TO newest_proposal_time_necro_at;
ALTER TABLE post RENAME COLUMN newest_comment_time_at TO newest_proposal_time_at;

-- 8. Rename category/site stat columns
ALTER TABLE category RENAME COLUMN comments TO proposals;
ALTER TABLE local_site RENAME COLUMN comments TO proposals;

-- 9. Rename rate limit columns
ALTER TABLE local_site_rate_limit RENAME COLUMN comment_max_requests TO proposal_max_requests;
ALTER TABLE local_site_rate_limit RENAME COLUMN comment_interval_seconds TO proposal_interval_seconds;

-- 10. Rename post_actions columns
ALTER TABLE post_actions RENAME COLUMN read_comments_at TO read_proposals_at;
ALTER TABLE post_actions RENAME COLUMN read_comments_amount TO read_proposals_amount;

-- 11. Rename local_user columns
ALTER TABLE local_user RENAME COLUMN default_comment_sort_type TO default_proposal_sort_type;
ALTER TABLE local_user RENAME COLUMN collapse_bot_comments TO collapse_bot_proposals;

-- 12. Rename local_site column
ALTER TABLE local_site RENAME COLUMN default_comment_sort_type TO default_proposal_sort_type;

-- 13. Rename indexes (IF EXISTS handles already-renamed indexes)
ALTER INDEX IF EXISTS idx_comment_creator RENAME TO idx_proposal_creator;
ALTER INDEX IF EXISTS idx_comment_post RENAME TO idx_proposal_post;
ALTER INDEX IF EXISTS idx_comment_published RENAME TO idx_proposal_published;
ALTER INDEX IF EXISTS idx_comment_language RENAME TO idx_proposal_language;
ALTER INDEX IF EXISTS idx_comment_content_trigram RENAME TO idx_proposal_content_trigram;
ALTER INDEX IF EXISTS idx_comment_controversy RENAME TO idx_proposal_controversy;
ALTER INDEX IF EXISTS idx_comment_hot RENAME TO idx_proposal_hot;
ALTER INDEX IF EXISTS idx_comment_nonzero_hotrank RENAME TO idx_proposal_nonzero_hotrank;
ALTER INDEX IF EXISTS idx_comment_score RENAME TO idx_proposal_score;
ALTER INDEX IF EXISTS idx_comment_actions_liked_not_null RENAME TO idx_proposal_actions_liked_not_null;
ALTER INDEX IF EXISTS idx_comment_actions_saved_not_null RENAME TO idx_proposal_actions_saved_not_null;
ALTER INDEX IF EXISTS idx_comment_actions_like_score RENAME TO idx_proposal_actions_like_score;
ALTER INDEX IF EXISTS idx_comment_actions_comment RENAME TO idx_proposal_actions_proposal;
ALTER INDEX IF EXISTS idx_comment_reply_comment RENAME TO idx_proposal_reply_proposal;
ALTER INDEX IF EXISTS idx_comment_reply_recipient RENAME TO idx_proposal_reply_recipient;
ALTER INDEX IF EXISTS idx_comment_reply_published RENAME TO idx_proposal_reply_published;
ALTER INDEX IF EXISTS idx_comment_report_published RENAME TO idx_proposal_report_published;
ALTER INDEX IF EXISTS idx_chat_room_current_comment_id RENAME TO idx_chat_room_current_proposal_id;
ALTER INDEX IF EXISTS idx_post_actions_read_comments_not_null RENAME TO idx_post_actions_read_proposals_not_null;
ALTER INDEX IF EXISTS idx_post_category_most_comments RENAME TO idx_post_category_most_proposals;
ALTER INDEX IF EXISTS idx_post_category_newest_comment_time RENAME TO idx_post_category_newest_proposal_time;
ALTER INDEX IF EXISTS idx_post_category_newest_comment_time_necro RENAME TO idx_post_category_newest_proposal_time_necro;
ALTER INDEX IF EXISTS idx_post_featured_category_most_comments RENAME TO idx_post_featured_category_most_proposals;
ALTER INDEX IF EXISTS idx_post_featured_category_newest_comment_time RENAME TO idx_post_featured_category_newest_proposal_time;
ALTER INDEX IF EXISTS idx_post_featured_category_newest_comment_time_necr RENAME TO idx_post_featured_category_newest_proposal_time_necr;
ALTER INDEX IF EXISTS idx_post_featured_local_most_comments RENAME TO idx_post_featured_local_most_proposals;
ALTER INDEX IF EXISTS idx_post_featured_local_newest_comment_time RENAME TO idx_post_featured_local_newest_proposal_time;
ALTER INDEX IF EXISTS idx_post_featured_local_newest_comment_time_necro RENAME TO idx_post_featured_local_newest_proposal_time_necro;
ALTER INDEX IF EXISTS idx_category_comments RENAME TO idx_category_proposals;

-- 13b. Rename constraint-backed indexes (pkey and unique constraints)
ALTER INDEX IF EXISTS admin_purge_comment_pkey RENAME TO admin_purge_proposal_pkey;
ALTER INDEX IF EXISTS mod_remove_comment_pkey RENAME TO mod_remove_proposal_pkey;
ALTER INDEX IF EXISTS comment_pkey RENAME TO proposal_pkey;
ALTER INDEX IF EXISTS comment_actions_pkey RENAME TO proposal_actions_pkey;
ALTER INDEX IF EXISTS comment_reply_pkey RENAME TO proposal_reply_pkey;
ALTER INDEX IF EXISTS comment_reply_recipient_id_comment_id_key RENAME TO proposal_reply_recipient_id_proposal_id_key;
ALTER INDEX IF EXISTS comment_report_pkey RENAME TO proposal_report_pkey;
ALTER INDEX IF EXISTS comment_report_comment_id_creator_id_key RENAME TO proposal_report_proposal_id_creator_id_key;
ALTER INDEX IF EXISTS inbox_combined_comment_reply_id_key RENAME TO inbox_combined_proposal_reply_id_key;
ALTER INDEX IF EXISTS inbox_combined_person_comment_mention_id_key RENAME TO inbox_combined_person_proposal_mention_id_key;
ALTER INDEX IF EXISTS modlog_combined_admin_purge_comment_id_key RENAME TO modlog_combined_admin_purge_proposal_id_key;
ALTER INDEX IF EXISTS modlog_combined_mod_remove_comment_id_key RENAME TO modlog_combined_mod_remove_proposal_id_key;
ALTER INDEX IF EXISTS person_content_combined_comment_id_key RENAME TO person_content_combined_proposal_id_key;
ALTER INDEX IF EXISTS person_liked_combined_person_id_comment_id_key RENAME TO person_liked_combined_person_id_proposal_id_key;
ALTER INDEX IF EXISTS person_mention_recipient_id_comment_id_key RENAME TO person_mention_recipient_id_proposal_id_key;
ALTER INDEX IF EXISTS person_saved_combined_person_id_comment_id_key RENAME TO person_saved_combined_person_id_proposal_id_key;
ALTER INDEX IF EXISTS report_combined_comment_report_id_key RENAME TO report_combined_proposal_report_id_key;
ALTER INDEX IF EXISTS search_combined_comment_id_key RENAME TO search_combined_proposal_id_key;

-- 14. Rename sequences
ALTER SEQUENCE IF EXISTS comment_id_seq RENAME TO proposal_id_seq;
ALTER SEQUENCE IF EXISTS comment_reply_id_seq RENAME TO proposal_reply_id_seq;
ALTER SEQUENCE IF EXISTS comment_report_id_seq RENAME TO proposal_report_id_seq;
ALTER SEQUENCE IF EXISTS person_comment_mention_id_seq RENAME TO person_proposal_mention_id_seq;
ALTER SEQUENCE IF EXISTS admin_purge_comment_id_seq RENAME TO admin_purge_proposal_id_seq;
ALTER SEQUENCE IF EXISTS mod_remove_comment_id_seq RENAME TO mod_remove_proposal_id_seq;

-- 15. Rename enum type
ALTER TYPE comment_sort_type_enum RENAME TO proposal_sort_type_enum;

-- 16. Re-add CHECK constraints with updated column names
ALTER TABLE modlog_combined ADD CONSTRAINT modlog_combined_check CHECK (
  (num_nonnulls(admin_allow_instance_id, admin_block_instance_id, admin_purge_proposal_id, admin_purge_category_id, admin_purge_person_id, admin_purge_post_id, mod_add_id, mod_add_category_id, mod_ban_id, mod_ban_from_category_id, mod_feature_post_id, mod_change_category_visibility_id, mod_lock_post_id, mod_remove_proposal_id, mod_remove_category_id, mod_remove_post_id, mod_transfer_category_id) = 1)
);

ALTER TABLE inbox_combined ADD CONSTRAINT inbox_combined_check CHECK (
  (num_nonnulls(proposal_reply_id, person_proposal_mention_id, person_post_mention_id) = 1)
);

ALTER TABLE report_combined ADD CONSTRAINT report_combined_check CHECK (
  (num_nonnulls(post_report_id, proposal_report_id, category_report_id) = 1)
);

ALTER TABLE search_combined ADD CONSTRAINT search_combined_check CHECK (
  (num_nonnulls(post_id, proposal_id, category_id, person_id) = 1)
);

ALTER TABLE person_content_combined ADD CONSTRAINT person_content_combined_check CHECK (
  (num_nonnulls(post_id, proposal_id) = 1)
);

ALTER TABLE person_saved_combined ADD CONSTRAINT person_saved_combined_check CHECK (
  (num_nonnulls(post_id, proposal_id) = 1)
);

ALTER TABLE person_liked_combined ADD CONSTRAINT person_liked_combined_check CHECK (
  (num_nonnulls(post_id, proposal_id) = 1)
);

-- 17. Rename enum values that contain 'comment'
DO $$ BEGIN
  ALTER TYPE post_sort_type_enum RENAME VALUE 'MostComments' TO 'MostProposals';
EXCEPTION WHEN invalid_parameter_value THEN NULL; END $$;
DO $$ BEGIN
  ALTER TYPE post_sort_type_enum RENAME VALUE 'NewComments' TO 'NewProposals';
EXCEPTION WHEN invalid_parameter_value THEN NULL; END $$;
DO $$ BEGIN
  ALTER TYPE post_notifications_mode_enum RENAME VALUE 'AllComments' TO 'AllProposals';
EXCEPTION WHEN invalid_parameter_value THEN NULL; END $$;
