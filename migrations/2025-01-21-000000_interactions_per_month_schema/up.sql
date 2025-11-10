-- Add the interactions_month column
ALTER TABLE category_aggregates
    ADD COLUMN interactions_month bigint NOT NULL DEFAULT 0;

