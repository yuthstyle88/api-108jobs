-- Remove currency support from withdraw_requests
DROP INDEX IF EXISTS idx_withdraw_requests_currency;

ALTER TABLE withdraw_requests DROP COLUMN IF EXISTS conversion_rate_used;
ALTER TABLE withdraw_requests DROP COLUMN IF EXISTS amount_currency;
ALTER TABLE withdraw_requests DROP COLUMN IF EXISTS currency_id;
