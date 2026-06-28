# Phase 5 — comment → proposal Full DB Rename

> **STATUS: PENDING APPROVAL** — do not execute until user approves the specific operation list.

## Overview

Full rename of the `comment` concept to `proposal` across DB, code, and routes.

## Operations Required (ordered)

### 1. Drop 7 CHECK constraints (BEFORE column renames)
```sql
ALTER TABLE modlog_combined DROP CONSTRAINT modlog_combined_check;
ALTER TABLE inbox_combined DROP CONSTRAINT inbox_combined_check;
ALTER TABLE report_combined DROP CONSTRAINT report_combined_check;
ALTER TABLE search_combined DROP CONSTRAINT search_combined_check;
ALTER TABLE person_content_combined DROP CONSTRAINT person_content_combined_check;
ALTER TABLE person_saved_combined DROP CONSTRAINT person_saved_combined_check;
ALTER TABLE person_liked_combined DROP CONSTRAINT person_liked_combined_check;
```

### 2. Rename 7 tables (leaf-first)
```sql
ALTER TABLE comment_actions RENAME TO proposal_actions;
ALTER TABLE comment_reply RENAME TO proposal_reply;
ALTER TABLE comment_report RENAME TO proposal_report;
ALTER TABLE admin_purge_comment RENAME TO admin_purge_proposal;
ALTER TABLE mod_remove_comment RENAME TO mod_remove_proposal;
ALTER TABLE person_comment_mention RENAME TO person_proposal_mention;
ALTER TABLE comment RENAME TO proposal;
```

### 3. Rename FK/counter columns in other tables
```sql
-- modlog_combined
ALTER TABLE modlog_combined RENAME COLUMN admin_purge_comment_id TO admin_purge_proposal_id;
ALTER TABLE modlog_combined RENAME COLUMN mod_remove_comment_id TO mod_remove_proposal_id;
-- report_combined
ALTER TABLE report_combined RENAME COLUMN comment_report_id TO proposal_report_id;
-- inbox_combined
ALTER TABLE inbox_combined RENAME COLUMN comment_reply_id TO proposal_reply_id;
ALTER TABLE inbox_combined RENAME COLUMN person_comment_mention_id TO person_proposal_mention_id;
-- combined views
ALTER TABLE person_content_combined RENAME COLUMN comment_id TO proposal_id;
ALTER TABLE person_saved_combined RENAME COLUMN comment_id TO proposal_id;
ALTER TABLE person_liked_combined RENAME COLUMN comment_id TO proposal_id;
ALTER TABLE search_combined RENAME COLUMN comment_id TO proposal_id;
-- business tables
ALTER TABLE billing RENAME COLUMN comment_id TO proposal_id;
ALTER TABLE chat_room RENAME COLUMN current_comment_id TO current_proposal_id;
ALTER TABLE delivery_details RENAME COLUMN linked_comment_id TO linked_proposal_id;
-- person stats
ALTER TABLE person RENAME COLUMN comment_count TO proposal_count;
ALTER TABLE person RENAME COLUMN comment_score TO proposal_score;
-- post counters
ALTER TABLE post RENAME COLUMN comments TO proposals;
ALTER TABLE post RENAME COLUMN newest_comment_time_necro_at TO newest_proposal_time_necro_at;
ALTER TABLE post RENAME COLUMN newest_comment_time_at TO newest_proposal_time_at;
-- other stats
ALTER TABLE category RENAME COLUMN comments TO proposals;
ALTER TABLE local_site RENAME COLUMN comments TO proposals;
-- rate limits
ALTER TABLE local_site_rate_limit RENAME COLUMN comment_max_requests TO proposal_max_requests;
ALTER TABLE local_site_rate_limit RENAME COLUMN comment_interval_seconds TO proposal_interval_seconds;
-- post_actions
ALTER TABLE post_actions RENAME COLUMN read_comments_at TO read_proposals_at;
ALTER TABLE post_actions RENAME COLUMN read_comments_amount TO read_proposals_amount;
```

### 4. Rename indexes (~15 index renames)
```sql
ALTER INDEX idx_comment_creator RENAME TO idx_proposal_creator;
ALTER INDEX idx_comment_post RENAME TO idx_proposal_post;
ALTER INDEX idx_comment_published RENAME TO idx_proposal_published;
ALTER INDEX idx_comment_language RENAME TO idx_proposal_language;
ALTER INDEX idx_comment_content_trigram RENAME TO idx_proposal_content_trigram;
ALTER INDEX idx_comment_controversy RENAME TO idx_proposal_controversy;
ALTER INDEX idx_comment_hot RENAME TO idx_proposal_hot;
ALTER INDEX idx_comment_nonzero_hotrank RENAME TO idx_proposal_nonzero_hotrank;
ALTER INDEX idx_comment_score RENAME TO idx_proposal_score;
ALTER INDEX idx_comment_actions_liked_not_null RENAME TO idx_proposal_actions_liked_not_null;
ALTER INDEX idx_comment_actions_saved_not_null RENAME TO idx_proposal_actions_saved_not_null;
ALTER INDEX idx_comment_actions_like_score RENAME TO idx_proposal_actions_like_score;
ALTER INDEX idx_comment_reply_comment RENAME TO idx_proposal_reply_proposal;
ALTER INDEX idx_comment_reply_recipient RENAME TO idx_proposal_reply_recipient;
ALTER INDEX idx_comment_reply_published RENAME TO idx_proposal_reply_published;
ALTER INDEX idx_comment_report_published RENAME TO idx_proposal_report_published;
ALTER INDEX idx_chat_room_current_comment_id RENAME TO idx_chat_room_current_proposal_id;
```

### 5. Rename sequences
```sql
ALTER SEQUENCE comment_id_seq RENAME TO proposal_id_seq;
ALTER SEQUENCE comment_reply_id_seq RENAME TO proposal_reply_id_seq;
ALTER SEQUENCE comment_report_id_seq RENAME TO proposal_report_id_seq;
ALTER SEQUENCE person_comment_mention_id_seq RENAME TO person_proposal_mention_id_seq;
```

### 6. Re-create CHECK constraints with updated column names
Must read current CHECK text from each combined table and replace column names.
See investigation report for exact SQL.

### 7. Rename enum type + dependent columns
```sql
ALTER TYPE comment_sort_type_enum RENAME TO proposal_sort_type_enum;
ALTER TABLE local_user RENAME COLUMN default_comment_sort_type TO default_proposal_sort_type;
ALTER TABLE local_site RENAME COLUMN default_comment_sort_type TO default_proposal_sort_type;
```

### 8. Update replaceable schema
`crates/db/replaceable_schema/triggers.sql` — all `'comment'` table name args, `comment_id` column refs, function names `r.comment_change_values()`, `r.search_combined_comment_score_update()` must be updated.

---

## ⚠️ CRITICAL RISK: ActivityPub URL

`r.comment_change_values()` in `triggers.sql` sets `ap_id = r.local_url('/comment/' || id)`.

**If this instance federates:** existing `ap_id` values stay as `/comment/...`, new ones get `/proposal/...`. Federated peers cannot resolve new proposals. 

**Decision required before proceeding:**
- [ ] Is this instance currently federating? (Phase 1 removed ActivityPub code but did it disable federation at the DB level?)
- [ ] Are there existing `ap_id` values in the comment/proposal table that remote instances reference?
- [ ] If yes: data migration to update all existing ap_ids + coordination with remote instances required

---

## Columns NOT Renamed (Decision Required)

- `delivery_rider_rating.comment` — free-text rating note → **leave as-is** (user-facing text field)
- `user_review.comment` — free-text review note → **leave as-is**
- `local_user.collapse_bot_comments` — UI setting name → **TBD: rename to collapse_bot_proposals?**

---

## Rust Code Changes (after migrations)

1. Run `diesel print-schema` to regenerate `crates/db/src/schema.rs`
2. Rename source files: `crates/db/src/source/comment*.rs` → `proposal*.rs`
3. Rename types: `Comment` → `Proposal`, `CommentId` → `ProposalId`, etc. (~20 types)
4. Rename `crates/db_views/comment/` → `crates/db_views/proposal/`
5. Mass rename in all 144 affected .rs files
6. Update `crates/api/api_common/src/comment.rs` → `proposal.rs`
7. Rename route functions: `create_comment` → `create_proposal`, etc.
8. Update API route paths: `/comment/*` → `/proposal/*`

## Flutter Client

Already abstracted as "Proposal" internally. Only API path strings need updating from `/comment` to `/proposal`.
