-- Note: PostgreSQL doesn't support removing enum values directly
-- This would require recreating the entire enum type
-- For now, we'll leave a comment explaining the situation
-- ALTER TYPE billing_status DROP VALUE 'RequestChange';
-- ALTER TYPE billing_status DROP VALUE 'Updated';

-- To properly rollback this migration, you would need to:
-- 1. Create a new enum without these values
-- 2. Update all references to use the new enum
-- 3. Drop the old enum
-- 4. Rename the new enum

SELECT 'Cannot automatically rollback enum values in PostgreSQL' as notice;