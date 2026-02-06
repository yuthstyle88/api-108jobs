-- Revert post.budget from Int4 back to Float8
ALTER TABLE post ALTER COLUMN budget TYPE FLOAT8 USING (budget::FLOAT8);
