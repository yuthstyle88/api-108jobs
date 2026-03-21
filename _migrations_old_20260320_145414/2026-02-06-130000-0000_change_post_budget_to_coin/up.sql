-- Convert post.budget from Float8 to Int4 to match Coin type (i32)
-- Coin represents money (smallest currency unit)
-- The data should already be stored as integer values (e.g., 10000 for $100.00)

-- Change the column type from Float8 to Int4
ALTER TABLE post ALTER COLUMN budget TYPE INTEGER USING (budget::INTEGER);
