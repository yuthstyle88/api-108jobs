-- Create wallet table for user balance management
CREATE TABLE wallet (
    id SERIAL PRIMARY KEY,
    balance FLOAT8 DEFAULT 0.0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ
);

ALTER TABLE local_user
  ADD COLUMN wallet_id INTEGER NOT NULL UNIQUE
    REFERENCES wallet(id)
    ON DELETE RESTRICT;

-- Create index for wallet lookups
CREATE INDEX idx_local_user_wallet_id ON local_user(wallet_id);