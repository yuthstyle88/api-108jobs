-- Wallet platform and triple-balance columns
DO $$
BEGIN
  CREATE TYPE tx_kind AS ENUM ('Deposit','Withdraw','Transfer');
EXCEPTION
  WHEN duplicate_object THEN NULL;
END
$$;


ALTER TABLE wallet
  ADD COLUMN IF NOT EXISTS is_platform boolean NOT NULL DEFAULT false,
  ADD COLUMN IF NOT EXISTS balance_total        float8 NOT NULL DEFAULT 0,
  ADD COLUMN IF NOT EXISTS balance_available    float8 NOT NULL DEFAULT 0,
  ADD COLUMN IF NOT EXISTS balance_outstanding  float8 NOT NULL DEFAULT 0;

-- Partial index for platform wallet
CREATE INDEX IF NOT EXISTS idx_wallet_platform
  ON wallet(is_platform) WHERE is_platform = true;


-- Wallet transactions (journal)
CREATE TABLE IF NOT EXISTS wallet_transaction (
  id               SERIAL PRIMARY KEY,
  wallet_id        INT NOT NULL REFERENCES wallet(id) ON DELETE CASCADE,
  reference_type   TEXT NOT NULL,
  reference_id     INT  NOT NULL,
  kind             tx_kind  DEFAULT 'Deposit' NOT NULL,
  amount           FLOAT8 NOT NULL CHECK (amount > 0),
  description      TEXT  NOT NULL,
  counter_user_id  INT,
  idempotency_key  TEXT NOT NULL,
  created_at       timestamptz NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_wallet_tx_idem
  ON wallet_transaction(idempotency_key, wallet_id);

CREATE INDEX IF NOT EXISTS idx_wallet_tx_wallet_time
  ON wallet_transaction(wallet_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_wallet_tx_ref_time
  ON wallet_transaction(reference_type, reference_id, created_at DESC);
