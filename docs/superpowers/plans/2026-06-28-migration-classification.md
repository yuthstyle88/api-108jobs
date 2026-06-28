# Migration Classification for Squash Planning

**Classified:** 281 migrations
**Status:** PENDING USER REVIEW before any deletion or squash

| Category | Count | Meaning |
|---|---|---|
| REQUIRED | 56 | Core 108Jobs business logic — must keep |
| COMPAT | 24 | Needed for DB FK compatibility — keep for now |
| LEMMY | 189 | Lemmy/fediverse/forum-only — candidates for removal |
| UNCLEAR | 12 | Ambiguous — needs human review before deciding |

> **Note:** 60 additional migrations listed in `ls` (341 total) vs 281 classified. Difference likely due to sub-directories or entries that couldn't be read. Do a manual sweep before squashing.

---

## REQUIRED (56) — Core 108Jobs Business Logic

```
2025-07-03-094254_chat_room                                | Creates chat_room + chat_participant tables
2025-07-03-094744_chat_message                             | Creates chat_message table
2025-07-28-085533_add_fields_to_post                       | Adds job_type, budget, deadline, intended_use, pending to post
2025-08-02-085207_add_wallet_table_and_wallet_id_to_person | Creates wallet table + person.wallet_id
2025-08-02-095035_add_escrow_and_billing_system            | Creates billing table (escrow)
2025-08-04-014547_add_otp_fields_to_local_user             | Adds shared_key/private_key to person (KYC/OTP)
2025-08-04-144518_add_bank_account_system                  | Creates banks + user_bank_accounts tables
2025-08-05-080012_add_veriffy_otp_field_to_local_site      | Adds verify_with_otp to local_site
2025-08-07-124013_add_fields_to_person                     | Adds contacts and skills columns to person
2025-08-09-091237_rename_to_pending_in_comment             | Renames federation_pending to pending in comment
2025-08-12-121900_wallet_platform_and_transactions         | Creates wallet_transaction table + wallet balances
2025-08-13-123530_coin                                     | Creates coin table
2025-08-13-181400_workflow_status_enum                     | Creates workflow table + workflow_status enum
2025-08-15-122800_create_job_budget_plan                   | Creates job_budget_plan table
2025-08-15-130000_add_coin_id_to_local_site                | Links local_site to coin
2025-08-18-064844_add_fields_to_person                     | Adds work_samples + portfolio_pics JSONB to person
2025-09-13-082957_add_room_id_to_workflow                  | Links workflow to chat_room; links chat_room to post
2025-09-15-042329-0000_create_review_user                  | Creates user_review table
2025-09-17-095200_add_current_comment_to_chat_room         | Adds current_comment_id to chat_room
2025-09-18-073957-0000_add_available_field_to_person       | Adds available flag to person
2025-09-18-202430_add_identity_cards                       | Creates identity_cards table (KYC)
2025-09-19-063900_add_deliverable_url_to_workflow          | Adds deliverable_url to workflow
2025-09-19-070410_add_active_to_workflow                   | Adds active flag to workflow
2025-09-24-145822-0000_add_status_before_cancel_to_workflow| Adds status_before_cancel to workflow
2025-09-24-160241-0000_add_room_id_to_billing              | Links billing to chat_room
2025-09-25-060356-0000_init_categories                     | Makes category nullable fields safe; seeds product categories
2025-09-25-060356-0026_add_field_billing_id_to_workflow    | Links workflow to billing
2025-09-27-042914-0000_create_last_reads_table             | Creates last_reads table (chat read tracking)
2025-10-23-042914-00335_add_field_local_user               | Adds accepted_terms to local_user
2025-10-26-090011-00111_create_pending_sender_ack          | Creates pending_sender_ack; adds sender_ack to chat_message
2025-11-05-131447-0000_create_table_top_up_requests        | Creates top_up_requests table
2025-11-07-052015-0000_add_is_secure_message_to_person     | Adds is_secure_message flag to person
2025-11-08-123706-0000_create_withdraw_requests            | Creates withdraw_requests table
2025-11-09-090009-001125_add_field_secure_chat_enabled     | Adds secure_chat_enabled to local_user
2025-11-10-091553-0000_make_is_default_bank_account_not_null | Makes is_default NOT NULL on user_bank_accounts
2025-12-12-031904-0000_add_last_message_id_to_chat_room    | Adds last_message_id/at to chat_room
2025-12-12-114802-0000_add_room_serial_id                  | Adds serial_id to chat_room for pagination
2025-12-19-060528-0000_create_table_chat_unread            | Creates chat_unread table
2026-01-09-040254-0000_create_table_rider                  | Creates rider table
2026-01-22-170000-0001_add_post_kind_and_delivery_status   | Adds post_kind enum to post
2026-01-22-170010-0002_create_table_delivery_details       | Creates delivery_details table
2026-01-22-170020-0003_create_delivery_location_tables     | Creates trip_location_current/history tables
2026-01-27-080000-0000_make_category_id_nullable           | Makes category_id nullable on post
2026-01-29-083309-0000_add_cancellation_reason_to_delivery_details | Adds cancellation_reason to delivery_details
2026-01-29-154835-0000_add_delivery_assignment_tracking    | Adds rider assignment tracking to delivery_details
2026-02-03-120000-0000_create_delivery_rider_rating        | Creates delivery_rider_rating table
2026-02-03-140000-0000_add_delivery_fee_to_delivery_details| Adds delivery_fee, wallet transaction ids to delivery_details
2026-02-06-130000-0000_change_post_budget_to_coin          | Converts post.budget from Float8 to Int4
2026-02-06-140000-0000_add_currency_support                | Creates currency, pricing_config, ride_session, ride_meter_snapshot
2026-02-09-150000-0000_add_currency_to_withdraw_requests   | Adds currency_id to withdraw_requests
2026-02-09-160000-0000_add_currency_to_top_up_requests     | Adds currency_id to top_up_requests
2026-02-09-170000-0000_seed_platform_assets                | Seeds platform wallet and 108JC coin
2026-02-12-091500-0001_add_ridetaxi_to_post_kind           | Adds RideTaxi to post_kind enum
2026-02-14-142800-rename-payment-method-enum               | Renames payment_method enum values
2026-02-26-100000-0000_add_passenger_contact_to_ride_session | Adds passenger_name/phone to ride_session
2026-03-10-000000-0000_add_cancellation_reason_to_ride_session | Adds cancellation_reason to ride_session
2026-03-24-100000-rename_delivery_status_to_trip_status    | Renames delivery_status enum to trip_status
2026-05-25-180000-0000_add_wallet_versioning_and_hold_ledger | Adds wallet.version + wallet_hold ledger
2026-06-27-000003-0000_chat_room_post_id_on_delete_set_null | Fixes FK on chat_room.post_id
2026-06-27-000005-0000_withdraw_status_cancelled           | Adds Cancelled to withdraw_status enum
```

---

## COMPAT (24) — Keep for DB Compatibility

```
00000000000000_diesel_initial_setup                        | Diesel ORM bootstrap — required
2021-03-09-171136_split_user_table_2                       | Renames user_ → person, creates local_user — core
2021-09-20-112945_jwt-secret                               | Creates secret table (jwt_secret) — auth
2022-10-06-183632_move_blocklist_to_db                     | Creates instance table — person.instance_id FK
2022-11-13-181529_create_taglines                          | Creates tagline table — in schema.rs
2023-07-11-084714_receive_activity_table                   | Creates sent_activity/received_activity — in schema.rs
2023-08-31-205559_add_image_upload                         | Creates image_upload (now local_image) — in schema.rs
2023-09-18-141700_login-token                              | Creates login_token table — auth
2024-02-24-034523_replaceable-schema                       | Pivots to replaceable schema; creates previously_run_sql
2024-05-05-162540_add_image_detail_table                   | Creates image_details — used for thumbnails
2024-09-16-174833_create_oauth_provider                    | Creates oauth_provider — Google/social login
2024-11-10-134311_smoosh-tables-together                   | Renames/merges action tables (post_actions, comment_actions)
2024-11-18-012113_custom_migration_runner                  | Creates previously_run_sql — migration runner
2024-12-02-181601_add_report_combined_table                | Creates report_combined — in schema.rs
2024-12-05-233704_add_person_content_combined_table        | Creates person_content/saved_combined — in schema.rs
2024-12-08-165614_add_modlog_combined_table                | Creates modlog_combined — in schema.rs
2024-12-10-193418_add_inbox_combined_table                 | Creates inbox_combined, person_post_mention — in schema.rs
2024-12-12-222846_add_search_combined_table                | Creates search_combined — in schema.rs
2025-03-04-105516_remove-aggregate-tables                  | Merges aggregate columns into comment/post/category/person
2025-06-08-084651_rename_timestamp_add_at                  | Mass rename of timestamp columns — broad impact
2026-06-27-000000-0000_add_tag_ap_id_default               | Fixes tag.ap_id DEFAULT — tag table in schema.rs
2026-06-27-000002-0000_fix_admin_allow_instance_published_at | Fixes missing column rename for admin_allow_instance
2026-06-27-000004-0000_advance_category_id_seq             | Fixes category sequence after seeding
```

---

## UNCLEAR (12) — Needs Human Decision

```
2019-12-11-181820_add_site_fields           | site table needed for local_site FK but columns are Lemmy-only
2021-09-20-112945_jwt-secret                | jwt_secret auth-critical but may be superseded
2021-11-23-132840_email_verification        | email_verification in schema.rs — 108Jobs uses OTP instead
2021-11-23-153753_add_invite_only_columns   | registration_application in schema.rs — needed for 108Jobs registration?
2023-02-11-173347_custom_emojis             | custom_emoji in schema.rs — does 108Jobs use emojis?
2023-07-24-232635_trigram-index             | Trigram indexes on comment/post — useful for 108Jobs search?
2023-08-02-144930_password-reset-token      | Renames token field — password reset needed for 108Jobs
2023-08-08-163911_add_post_listing_mode_setting | post_listing_mode_enum in local_user — does Flutter use it?
2023-09-11-110040_rework-2fa-setup          | TOTP 2FA rework — 108Jobs has OTP, may not want Lemmy 2FA
2024-05-04-140749_separate_triggers         | No-op, triggers replaceable schema re-run
2024-12-20-090225_update-replaceable-schema | No-op, triggers replaceable schema re-run
2025-07-28-085533_add_fields_to_post        | REQUIRED for jobs but overlaps federation_pending rename chain
```

---

## 🚨 GAPS FOUND

1. **`skills` and `certificates` tables** appear in `schema.rs` but have NO migration. Either created manually or in code. Must write a migration or remove from schema.rs before squashing.

2. **60 migrations unclassified** — the `ls` count is 341 but only 281 were read. Manual audit needed for the remaining 60 before squash.

---

## Squash Strategy

**Phase 1 (safe now):** Remove the 189 LEMMY migrations from new installs only. Keep them in the directory for rollback. New baseline starts at `2025-07-03_chat_room` (first 108Jobs-specific migration).

**Phase 2 (after Phase 4 Lemmy removal):** Once Lemmy code is gone and tables verified unused, drop the LEMMY tables from the DB with a single new migration. Then squash all pre-2025 migrations into a baseline.

**Phase 3 (pre-prod):** Resolve the 12 UNCLEAR items, fix the `skills`/`certificates` gap, write the baseline squash migration.
