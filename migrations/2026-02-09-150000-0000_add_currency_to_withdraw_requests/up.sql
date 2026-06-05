-- Add currency support to withdraw_requests
-- Users can select which currency to withdraw to, and the conversion rate is recorded

-- Add currency_id column (references currency table)
ALTER TABLE withdraw_requests
ADD COLUMN currency_id INTEGER NOT NULL DEFAULT 1 REFERENCES currency(id);

-- Add amount_currency - the amount in the selected currency (calculated at withdrawal time)
-- Using FLOAT8 for simplicity in Rust mapping (f64)
ALTER TABLE withdraw_requests
ADD COLUMN amount_currency FLOAT8 NOT NULL DEFAULT 0;

-- Add conversion_rate_used - the rate used to convert Coins to currency (for audit trail)
ALTER TABLE withdraw_requests
ADD COLUMN conversion_rate_used INTEGER NOT NULL DEFAULT 1;

-- Create index for faster lookups by currency
CREATE INDEX idx_withdraw_requests_currency ON withdraw_requests(currency_id);
