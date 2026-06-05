-- Revert currency support changes for top_up_requests
DROP INDEX IF EXISTS idx_top_up_requests_currency;

-- Restore currency_name column from currency_id
ALTER TABLE top_up_requests ADD COLUMN currency_name TEXT NOT NULL DEFAULT 'THB';

-- Migrate back: currency_id to currency_name
UPDATE top_up_requests SET currency_name = (
  CASE currency_id
    WHEN 1 THEN 'THB'
    WHEN 2 THEN 'IDR'
    WHEN 3 THEN 'VND'
    ELSE 'THB'
  END
);

-- Remove new columns
ALTER TABLE top_up_requests DROP COLUMN IF EXISTS conversion_rate_used;
ALTER TABLE top_up_requests DROP COLUMN IF EXISTS amount_coin;
ALTER TABLE top_up_requests DROP COLUMN IF EXISTS currency_id;
