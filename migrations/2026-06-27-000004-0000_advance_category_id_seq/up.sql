-- Migration 2025-09-25-060356-0000_init_categories seeds the `category` table
-- with explicit ids (1..50) but never advances `category_id_seq`. The sequence
-- therefore still points inside the seeded range, so the next
-- `Category::create` (which relies on the serial default) hands out an id that
-- already exists and fails with `duplicate key value violates unique
-- constraint "category_pkey"` — in production category creation as well as in
-- several db_views tests (search_combined, vote).
--
-- Advance the sequence past the highest seeded id. Idempotent and additive;
-- GREATEST(..., 1) keeps setval valid even if the table is empty.
SELECT
    setval('category_id_seq', GREATEST(COALESCE((SELECT max(id) FROM category), 0), 1));
