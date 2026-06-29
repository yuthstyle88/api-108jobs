ALTER TABLE ONLY public.admin_allow_instance
    ADD CONSTRAINT admin_allow_instance_admin_person_id_fkey FOREIGN KEY (admin_person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.admin_allow_instance
    ADD CONSTRAINT admin_allow_instance_instance_id_fkey FOREIGN KEY (instance_id) REFERENCES public.instance(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.admin_block_instance
    ADD CONSTRAINT admin_block_instance_admin_person_id_fkey FOREIGN KEY (admin_person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.admin_block_instance
    ADD CONSTRAINT admin_block_instance_instance_id_fkey FOREIGN KEY (instance_id) REFERENCES public.instance(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.admin_purge_category
    ADD CONSTRAINT admin_purge_category_admin_person_id_fkey FOREIGN KEY (admin_person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.admin_purge_proposal
    ADD CONSTRAINT admin_purge_comment_admin_person_id_fkey FOREIGN KEY (admin_person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.admin_purge_proposal
    ADD CONSTRAINT admin_purge_comment_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.post(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.admin_purge_person
    ADD CONSTRAINT admin_purge_person_admin_person_id_fkey FOREIGN KEY (admin_person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.admin_purge_post
    ADD CONSTRAINT admin_purge_post_admin_person_id_fkey FOREIGN KEY (admin_person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.admin_purge_post
    ADD CONSTRAINT admin_purge_post_category_id_fkey FOREIGN KEY (category_id) REFERENCES public.category(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.billing
    ADD CONSTRAINT billing_comment_id_fkey FOREIGN KEY (proposal_id) REFERENCES public.proposal(id) ON UPDATE CASCADE ON DELETE SET NULL;

ALTER TABLE ONLY public.billing
    ADD CONSTRAINT billing_employer_id_fkey FOREIGN KEY (employer_id) REFERENCES public.local_user(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.billing
    ADD CONSTRAINT billing_freelancer_id_fkey FOREIGN KEY (freelancer_id) REFERENCES public.local_user(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.billing
    ADD CONSTRAINT billing_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.post(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.billing
    ADD CONSTRAINT billing_room_id_fkey FOREIGN KEY (room_id) REFERENCES public.chat_room(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.category_actions
    ADD CONSTRAINT category_actions_category_id_fkey FOREIGN KEY (category_id) REFERENCES public.category(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.category_actions
    ADD CONSTRAINT category_actions_follow_approver_id_fkey FOREIGN KEY (follow_approver_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.category_actions
    ADD CONSTRAINT category_actions_person_id_fkey FOREIGN KEY (person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.category_language
    ADD CONSTRAINT category_language_category_id_fkey FOREIGN KEY (category_id) REFERENCES public.category(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.category_language
    ADD CONSTRAINT category_language_language_id_fkey FOREIGN KEY (language_id) REFERENCES public.language(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.category_report
    ADD CONSTRAINT category_report_category_id_fkey FOREIGN KEY (category_id) REFERENCES public.category(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.category_report
    ADD CONSTRAINT category_report_creator_id_fkey FOREIGN KEY (creator_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.category_report
    ADD CONSTRAINT category_report_resolver_id_fkey FOREIGN KEY (resolver_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.chat_message
    ADD CONSTRAINT chat_message_receiver_id_fkey FOREIGN KEY (receiver_id) REFERENCES public.local_user(id) ON UPDATE CASCADE ON DELETE SET NULL;

ALTER TABLE ONLY public.chat_message
    ADD CONSTRAINT chat_message_room_id_fkey FOREIGN KEY (room_id) REFERENCES public.chat_room(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.chat_message
    ADD CONSTRAINT chat_message_sender_id_fkey FOREIGN KEY (sender_id) REFERENCES public.local_user(id) ON UPDATE CASCADE ON DELETE SET NULL;

ALTER TABLE ONLY public.chat_participant
    ADD CONSTRAINT chat_participant_member_id_fkey FOREIGN KEY (member_id) REFERENCES public.local_user(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.chat_participant
    ADD CONSTRAINT chat_participant_room_id_fkey FOREIGN KEY (room_id) REFERENCES public.chat_room(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.chat_room
    ADD CONSTRAINT chat_room_current_comment_id_fkey FOREIGN KEY (current_proposal_id) REFERENCES public.proposal(id) ON UPDATE CASCADE ON DELETE SET NULL;

ALTER TABLE ONLY public.chat_room
    ADD CONSTRAINT chat_room_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.post(id) ON UPDATE CASCADE ON DELETE SET NULL;

ALTER TABLE ONLY public.chat_unread
    ADD CONSTRAINT chat_unread_local_user_id_fkey FOREIGN KEY (local_user_id) REFERENCES public.local_user(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.chat_unread
    ADD CONSTRAINT chat_unread_room_id_fkey FOREIGN KEY (room_id) REFERENCES public.chat_room(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.proposal_actions
    ADD CONSTRAINT proposal_actions_proposal_id_fkey FOREIGN KEY (proposal_id) REFERENCES public.proposal(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.proposal_actions
    ADD CONSTRAINT comment_actions_person_id_fkey FOREIGN KEY (person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.proposal
    ADD CONSTRAINT comment_creator_id_fkey FOREIGN KEY (creator_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.proposal
    ADD CONSTRAINT comment_language_id_fkey FOREIGN KEY (language_id) REFERENCES public.language(id);

ALTER TABLE ONLY public.proposal
    ADD CONSTRAINT comment_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.post(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.proposal_reply
    ADD CONSTRAINT comment_reply_comment_id_fkey FOREIGN KEY (comment_id) REFERENCES public.proposal(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.proposal_reply
    ADD CONSTRAINT comment_reply_recipient_id_fkey FOREIGN KEY (recipient_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.proposal_report
    ADD CONSTRAINT comment_report_comment_id_fkey FOREIGN KEY (comment_id) REFERENCES public.proposal(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.proposal_report
    ADD CONSTRAINT comment_report_creator_id_fkey FOREIGN KEY (creator_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.proposal_report
    ADD CONSTRAINT comment_report_resolver_id_fkey FOREIGN KEY (resolver_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.currency_rate_history
    ADD CONSTRAINT currency_rate_history_changed_by_fkey FOREIGN KEY (changed_by) REFERENCES public.local_user(id);

ALTER TABLE ONLY public.currency_rate_history
    ADD CONSTRAINT currency_rate_history_currency_id_fkey FOREIGN KEY (currency_id) REFERENCES public.currency(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.currency
    ADD CONSTRAINT currency_rate_last_updated_by_fkey FOREIGN KEY (rate_last_updated_by) REFERENCES public.local_user(id);

ALTER TABLE ONLY public.custom_emoji_keyword
    ADD CONSTRAINT custom_emoji_keyword_custom_emoji_id_fkey FOREIGN KEY (custom_emoji_id) REFERENCES public.custom_emoji(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.delivery_details
    ADD CONSTRAINT delivery_details_assigned_by_person_id_fkey FOREIGN KEY (assigned_by_person_id) REFERENCES public.person(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.delivery_details
    ADD CONSTRAINT delivery_details_assigned_rider_id_fkey FOREIGN KEY (assigned_rider_id) REFERENCES public.rider(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.delivery_details
    ADD CONSTRAINT delivery_details_linked_comment_id_fkey FOREIGN KEY (linked_proposal_id) REFERENCES public.proposal(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.delivery_details
    ADD CONSTRAINT delivery_details_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.post(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.delivery_rider_rating
    ADD CONSTRAINT delivery_rider_rating_employer_id_fkey FOREIGN KEY (employer_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.delivery_rider_rating
    ADD CONSTRAINT delivery_rider_rating_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.post(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.delivery_rider_rating
    ADD CONSTRAINT delivery_rider_rating_rider_id_fkey FOREIGN KEY (rider_id) REFERENCES public.rider(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.email_verification
    ADD CONSTRAINT email_verification_local_user_id_fkey FOREIGN KEY (local_user_id) REFERENCES public.local_user(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.inbox_combined
    ADD CONSTRAINT inbox_combined_comment_reply_id_fkey FOREIGN KEY (proposal_reply_id) REFERENCES public.proposal_reply(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.inbox_combined
    ADD CONSTRAINT inbox_combined_person_comment_mention_id_fkey FOREIGN KEY (person_proposal_mention_id) REFERENCES public.person_proposal_mention(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.inbox_combined
    ADD CONSTRAINT inbox_combined_person_post_mention_id_fkey FOREIGN KEY (person_post_mention_id) REFERENCES public.person_post_mention(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.instance_actions
    ADD CONSTRAINT instance_actions_instance_id_fkey FOREIGN KEY (instance_id) REFERENCES public.instance(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.instance_actions
    ADD CONSTRAINT instance_actions_person_id_fkey FOREIGN KEY (person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.job_budget_plan
    ADD CONSTRAINT job_budget_plan_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.post(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.last_reads
    ADD CONSTRAINT last_reads_local_user_id_fkey FOREIGN KEY (local_user_id) REFERENCES public.local_user(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.last_reads
    ADD CONSTRAINT last_reads_room_id_fkey FOREIGN KEY (room_id) REFERENCES public.chat_room(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.local_image
    ADD CONSTRAINT local_image_person_id_fkey FOREIGN KEY (person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.local_image
    ADD CONSTRAINT local_image_thumbnail_for_post_id_fkey FOREIGN KEY (thumbnail_for_post_id) REFERENCES public.post(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.local_site
    ADD CONSTRAINT local_site_coin_id_fkey FOREIGN KEY (coin_id) REFERENCES public.coin(id);

ALTER TABLE ONLY public.local_site_rate_limit
    ADD CONSTRAINT local_site_rate_limit_local_site_id_fkey FOREIGN KEY (local_site_id) REFERENCES public.local_site(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.local_site
    ADD CONSTRAINT local_site_site_id_fkey FOREIGN KEY (site_id) REFERENCES public.site(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.local_user_keyword_block
    ADD CONSTRAINT local_user_keyword_block_local_user_id_fkey FOREIGN KEY (local_user_id) REFERENCES public.local_user(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.local_user_language
    ADD CONSTRAINT local_user_language_language_id_fkey FOREIGN KEY (language_id) REFERENCES public.language(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.local_user_language
    ADD CONSTRAINT local_user_language_local_user_id_fkey FOREIGN KEY (local_user_id) REFERENCES public.local_user(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.local_user
    ADD CONSTRAINT local_user_person_id_fkey FOREIGN KEY (person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.login_token
    ADD CONSTRAINT login_token_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.local_user(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.mod_add_category
    ADD CONSTRAINT mod_add_category_category_id_fkey FOREIGN KEY (category_id) REFERENCES public.category(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.mod_add_category
    ADD CONSTRAINT mod_add_category_mod_person_id_fkey FOREIGN KEY (mod_person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.mod_add_category
    ADD CONSTRAINT mod_add_category_other_person_id_fkey FOREIGN KEY (other_person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.mod_add
    ADD CONSTRAINT mod_add_mod_person_id_fkey FOREIGN KEY (mod_person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.mod_add
    ADD CONSTRAINT mod_add_other_person_id_fkey FOREIGN KEY (other_person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.mod_ban_from_category
    ADD CONSTRAINT mod_ban_from_category_category_id_fkey FOREIGN KEY (category_id) REFERENCES public.category(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.mod_ban_from_category
    ADD CONSTRAINT mod_ban_from_category_mod_person_id_fkey FOREIGN KEY (mod_person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.mod_ban_from_category
    ADD CONSTRAINT mod_ban_from_category_other_person_id_fkey FOREIGN KEY (other_person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.mod_ban
    ADD CONSTRAINT mod_ban_instance_id_fkey FOREIGN KEY (instance_id) REFERENCES public.instance(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.mod_ban
    ADD CONSTRAINT mod_ban_mod_person_id_fkey FOREIGN KEY (mod_person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.mod_ban
    ADD CONSTRAINT mod_ban_other_person_id_fkey FOREIGN KEY (other_person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.mod_change_category_visibility
    ADD CONSTRAINT mod_change_category_visibility_category_id_fkey FOREIGN KEY (category_id) REFERENCES public.category(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.mod_change_category_visibility
    ADD CONSTRAINT mod_change_category_visibility_mod_person_id_fkey FOREIGN KEY (mod_person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.mod_lock_post
    ADD CONSTRAINT mod_lock_post_mod_person_id_fkey FOREIGN KEY (mod_person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.mod_lock_post
    ADD CONSTRAINT mod_lock_post_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.post(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.mod_remove_category
    ADD CONSTRAINT mod_remove_category_category_id_fkey FOREIGN KEY (category_id) REFERENCES public.category(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.mod_remove_category
    ADD CONSTRAINT mod_remove_category_mod_person_id_fkey FOREIGN KEY (mod_person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.mod_remove_proposal
    ADD CONSTRAINT mod_remove_comment_comment_id_fkey FOREIGN KEY (comment_id) REFERENCES public.proposal(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.mod_remove_proposal
    ADD CONSTRAINT mod_remove_comment_mod_person_id_fkey FOREIGN KEY (mod_person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.mod_remove_post
    ADD CONSTRAINT mod_remove_post_mod_person_id_fkey FOREIGN KEY (mod_person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.mod_remove_post
    ADD CONSTRAINT mod_remove_post_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.post(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.mod_feature_post
    ADD CONSTRAINT mod_sticky_post_mod_person_id_fkey FOREIGN KEY (mod_person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.mod_feature_post
    ADD CONSTRAINT mod_sticky_post_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.post(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.mod_transfer_category
    ADD CONSTRAINT mod_transfer_category_category_id_fkey FOREIGN KEY (category_id) REFERENCES public.category(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.mod_transfer_category
    ADD CONSTRAINT mod_transfer_category_mod_person_id_fkey FOREIGN KEY (mod_person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.mod_transfer_category
    ADD CONSTRAINT mod_transfer_category_other_person_id_fkey FOREIGN KEY (other_person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_admin_allow_instance_id_fkey FOREIGN KEY (admin_allow_instance_id) REFERENCES public.admin_allow_instance(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_admin_block_instance_id_fkey FOREIGN KEY (admin_block_instance_id) REFERENCES public.admin_block_instance(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_admin_purge_category_id_fkey FOREIGN KEY (admin_purge_category_id) REFERENCES public.admin_purge_category(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_admin_purge_comment_id_fkey FOREIGN KEY (admin_purge_proposal_id) REFERENCES public.admin_purge_proposal(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_admin_purge_person_id_fkey FOREIGN KEY (admin_purge_person_id) REFERENCES public.admin_purge_person(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_admin_purge_post_id_fkey FOREIGN KEY (admin_purge_post_id) REFERENCES public.admin_purge_post(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_mod_add_category_id_fkey FOREIGN KEY (mod_add_category_id) REFERENCES public.mod_add_category(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_mod_add_id_fkey FOREIGN KEY (mod_add_id) REFERENCES public.mod_add(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_mod_ban_from_category_id_fkey FOREIGN KEY (mod_ban_from_category_id) REFERENCES public.mod_ban_from_category(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_mod_ban_id_fkey FOREIGN KEY (mod_ban_id) REFERENCES public.mod_ban(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_mod_change_category_visibility_id_fkey FOREIGN KEY (mod_change_category_visibility_id) REFERENCES public.mod_change_category_visibility(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_mod_feature_post_id_fkey FOREIGN KEY (mod_feature_post_id) REFERENCES public.mod_feature_post(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_mod_lock_post_id_fkey FOREIGN KEY (mod_lock_post_id) REFERENCES public.mod_lock_post(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_mod_remove_category_id_fkey FOREIGN KEY (mod_remove_category_id) REFERENCES public.mod_remove_category(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_mod_remove_comment_id_fkey FOREIGN KEY (mod_remove_proposal_id) REFERENCES public.mod_remove_proposal(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_mod_remove_post_id_fkey FOREIGN KEY (mod_remove_post_id) REFERENCES public.mod_remove_post(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.modlog_combined
    ADD CONSTRAINT modlog_combined_mod_transfer_category_id_fkey FOREIGN KEY (mod_transfer_category_id) REFERENCES public.mod_transfer_category(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.oauth_account
    ADD CONSTRAINT oauth_account_local_user_id_fkey FOREIGN KEY (local_user_id) REFERENCES public.local_user(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.oauth_account
    ADD CONSTRAINT oauth_account_oauth_provider_id_fkey FOREIGN KEY (oauth_provider_id) REFERENCES public.oauth_provider(id) ON UPDATE CASCADE ON DELETE RESTRICT;

ALTER TABLE ONLY public.password_reset_request
    ADD CONSTRAINT password_reset_request_local_user_id_fkey FOREIGN KEY (local_user_id) REFERENCES public.local_user(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.person_actions
    ADD CONSTRAINT person_actions_person_id_fkey FOREIGN KEY (person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.person_actions
    ADD CONSTRAINT person_actions_target_id_fkey FOREIGN KEY (target_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.person_content_combined
    ADD CONSTRAINT person_content_combined_comment_id_fkey FOREIGN KEY (proposal_id) REFERENCES public.proposal(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.person_content_combined
    ADD CONSTRAINT person_content_combined_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.post(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.person
    ADD CONSTRAINT person_instance_id_fkey FOREIGN KEY (instance_id) REFERENCES public.instance(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.person_liked_combined
    ADD CONSTRAINT person_liked_combined_comment_id_fkey FOREIGN KEY (proposal_id) REFERENCES public.proposal(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.person_liked_combined
    ADD CONSTRAINT person_liked_combined_person_id_fkey FOREIGN KEY (person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.person_liked_combined
    ADD CONSTRAINT person_liked_combined_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.post(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.person_proposal_mention
    ADD CONSTRAINT person_mention_comment_id_fkey FOREIGN KEY (comment_id) REFERENCES public.proposal(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.person_proposal_mention
    ADD CONSTRAINT person_mention_recipient_id_fkey FOREIGN KEY (recipient_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.person_post_mention
    ADD CONSTRAINT person_post_mention_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.post(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.person_post_mention
    ADD CONSTRAINT person_post_mention_recipient_id_fkey FOREIGN KEY (recipient_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.person_saved_combined
    ADD CONSTRAINT person_saved_combined_comment_id_fkey FOREIGN KEY (proposal_id) REFERENCES public.proposal(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.person_saved_combined
    ADD CONSTRAINT person_saved_combined_person_id_fkey FOREIGN KEY (person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.person_saved_combined
    ADD CONSTRAINT person_saved_combined_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.post(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.person
    ADD CONSTRAINT person_wallet_id_fkey FOREIGN KEY (wallet_id) REFERENCES public.wallet(id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.post_actions
    ADD CONSTRAINT post_actions_person_id_fkey FOREIGN KEY (person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.post_actions
    ADD CONSTRAINT post_actions_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.post(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.post
    ADD CONSTRAINT post_category_id_fkey FOREIGN KEY (category_id) REFERENCES public.category(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.post
    ADD CONSTRAINT post_creator_id_fkey FOREIGN KEY (creator_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.post
    ADD CONSTRAINT post_language_id_fkey FOREIGN KEY (language_id) REFERENCES public.language(id);

ALTER TABLE ONLY public.post_report
    ADD CONSTRAINT post_report_creator_id_fkey FOREIGN KEY (creator_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.post_report
    ADD CONSTRAINT post_report_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.post(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.post_report
    ADD CONSTRAINT post_report_resolver_id_fkey FOREIGN KEY (resolver_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.post_tag
    ADD CONSTRAINT post_tag_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.post(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.post_tag
    ADD CONSTRAINT post_tag_tag_id_fkey FOREIGN KEY (tag_id) REFERENCES public.tag(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.pricing_config
    ADD CONSTRAINT pricing_config_currency_id_fkey FOREIGN KEY (currency_id) REFERENCES public.currency(id);

ALTER TABLE ONLY public.registration_application
    ADD CONSTRAINT registration_application_admin_id_fkey FOREIGN KEY (admin_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.registration_application
    ADD CONSTRAINT registration_application_local_user_id_fkey FOREIGN KEY (local_user_id) REFERENCES public.local_user(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.report_combined
    ADD CONSTRAINT report_combined_category_report_id_fkey FOREIGN KEY (category_report_id) REFERENCES public.category_report(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.report_combined
    ADD CONSTRAINT report_combined_comment_report_id_fkey FOREIGN KEY (proposal_report_id) REFERENCES public.proposal_report(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.report_combined
    ADD CONSTRAINT report_combined_post_report_id_fkey FOREIGN KEY (post_report_id) REFERENCES public.post_report(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.ride_meter_snapshot
    ADD CONSTRAINT ride_meter_snapshot_ride_session_id_fkey FOREIGN KEY (ride_session_id) REFERENCES public.ride_session(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.ride_session
    ADD CONSTRAINT ride_session_employer_id_fkey FOREIGN KEY (employer_id) REFERENCES public.local_user(id);

ALTER TABLE ONLY public.ride_session
    ADD CONSTRAINT ride_session_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.post(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.ride_session
    ADD CONSTRAINT ride_session_pricing_config_id_fkey FOREIGN KEY (pricing_config_id) REFERENCES public.pricing_config(id);

ALTER TABLE ONLY public.ride_session
    ADD CONSTRAINT ride_session_rider_id_fkey FOREIGN KEY (rider_id) REFERENCES public.rider(id) ON DELETE SET NULL;

ALTER TABLE ONLY public.rider
    ADD CONSTRAINT rider_person_id_fkey FOREIGN KEY (person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.rider
    ADD CONSTRAINT rider_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.local_user(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.search_combined
    ADD CONSTRAINT search_combined_category_id_fkey FOREIGN KEY (category_id) REFERENCES public.category(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.search_combined
    ADD CONSTRAINT search_combined_comment_id_fkey FOREIGN KEY (proposal_id) REFERENCES public.proposal(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.search_combined
    ADD CONSTRAINT search_combined_person_id_fkey FOREIGN KEY (person_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.search_combined
    ADD CONSTRAINT search_combined_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.post(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.site
    ADD CONSTRAINT site_instance_id_fkey FOREIGN KEY (instance_id) REFERENCES public.instance(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.site_language
    ADD CONSTRAINT site_language_language_id_fkey FOREIGN KEY (language_id) REFERENCES public.language(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.site_language
    ADD CONSTRAINT site_language_site_id_fkey FOREIGN KEY (site_id) REFERENCES public.site(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.tag
    ADD CONSTRAINT tag_category_id_fkey FOREIGN KEY (category_id) REFERENCES public.category(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.top_up_requests
    ADD CONSTRAINT top_up_requests_currency_id_fkey FOREIGN KEY (currency_id) REFERENCES public.currency(id);

ALTER TABLE ONLY public.top_up_requests
    ADD CONSTRAINT top_up_requests_local_user_id_fkey FOREIGN KEY (local_user_id) REFERENCES public.local_user(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.trip_location_current
    ADD CONSTRAINT trip_location_current_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.post(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.trip_location_current
    ADD CONSTRAINT trip_location_current_rider_id_fkey FOREIGN KEY (rider_id) REFERENCES public.rider(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.trip_location_history
    ADD CONSTRAINT trip_location_history_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.post(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.trip_location_history
    ADD CONSTRAINT trip_location_history_rider_id_fkey FOREIGN KEY (rider_id) REFERENCES public.rider(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.user_bank_accounts
    ADD CONSTRAINT user_bank_accounts_bank_id_fkey FOREIGN KEY (bank_id) REFERENCES public.banks(id);

ALTER TABLE ONLY public.user_bank_accounts
    ADD CONSTRAINT user_bank_accounts_local_user_id_fkey FOREIGN KEY (local_user_id) REFERENCES public.local_user(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.user_review
    ADD CONSTRAINT user_review_reviewee_id_fkey FOREIGN KEY (reviewee_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.user_review
    ADD CONSTRAINT user_review_reviewer_id_fkey FOREIGN KEY (reviewer_id) REFERENCES public.person(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.user_review
    ADD CONSTRAINT user_review_workflow_id_fkey FOREIGN KEY (workflow_id) REFERENCES public.workflow(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE ONLY public.wallet_hold
    ADD CONSTRAINT wallet_hold_billing_id_fkey FOREIGN KEY (billing_id) REFERENCES public.billing(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.wallet_hold
    ADD CONSTRAINT wallet_hold_wallet_id_fkey FOREIGN KEY (wallet_id) REFERENCES public.wallet(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.wallet_transaction
    ADD CONSTRAINT wallet_transaction_wallet_id_fkey FOREIGN KEY (wallet_id) REFERENCES public.wallet(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.withdraw_requests
    ADD CONSTRAINT withdraw_requests_currency_id_fkey FOREIGN KEY (currency_id) REFERENCES public.currency(id);

ALTER TABLE ONLY public.withdraw_requests
    ADD CONSTRAINT withdraw_requests_local_user_id_fkey FOREIGN KEY (local_user_id) REFERENCES public.local_user(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.withdraw_requests
    ADD CONSTRAINT withdraw_requests_user_bank_account_id_fkey FOREIGN KEY (user_bank_account_id) REFERENCES public.user_bank_accounts(id) ON DELETE RESTRICT;

ALTER TABLE ONLY public.withdraw_requests
    ADD CONSTRAINT withdraw_requests_wallet_id_fkey FOREIGN KEY (wallet_id) REFERENCES public.wallet(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.workflow
    ADD CONSTRAINT workflow_billing_id_fkey FOREIGN KEY (billing_id) REFERENCES public.billing(id);

ALTER TABLE ONLY public.workflow
    ADD CONSTRAINT workflow_post_id_fkey FOREIGN KEY (post_id) REFERENCES public.post(id) ON DELETE CASCADE;

ALTER TABLE ONLY public.workflow
    ADD CONSTRAINT workflow_room_id_fkey FOREIGN KEY (room_id) REFERENCES public.chat_room(id) ON UPDATE CASCADE ON DELETE CASCADE;

--
-- PostgreSQL database dump complete
--
