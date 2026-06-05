-- Add currency support to top_up_requests
-- Track currency conversion when users top up via SCB

-- Add currency_id column (references currency table)
ALTER TABLE top_up_requests
ADD COLUMN currency_id INTEGER NOT NULL DEFAULT 1 REFERENCES currency(id);

-- Add amount_coin - the converted amount in Coins (added to wallet)
ALTER TABLE top_up_requests
ADD COLUMN amount_coin INTEGER NOT NULL DEFAULT 0;

-- Add conversion_rate_used - the rate used to convert currency to Coins (for audit trail)
ALTER TABLE top_up_requests
ADD COLUMN conversion_rate_used INTEGER NOT NULL DEFAULT 1;

-- Migrate existing currency_name to currency_id (basic mapping, admin should verify)
UPDATE top_up_requests SET currency_id = (
  CASE LOWER(currency_name)
    WHEN 'thb' THEN 1
    WHEN 'idr' THEN 2  -- Assuming ID will be 2
    WHEN 'vnd' THEN 3  -- Assuming VND will be 3
    ELSE 1  -- Default to THB
  END
) WHERE currency_id = 1;

-- Calculate amount_coin from existing amount using the conversion rate
-- For existing records, we'll assume rate=1 for THB, admin can adjust if needed
UPDATE top_up_requests SET amount_coin = CAST(amount AS INTEGER) * conversion_rate_used WHERE amount_coin = 0;

-- Drop the old currency_name column (data migrated to currency_id)
ALTER TABLE top_up_requests DROP COLUMN currency_name;

-- Create index for faster lookups by currency
CREATE INDEX idx_top_up_requests_currency ON top_up_requests(currency_id);
