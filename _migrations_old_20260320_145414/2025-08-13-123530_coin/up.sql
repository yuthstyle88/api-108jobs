-- Create coin table for platform coin metadata
CREATE TABLE IF NOT EXISTS coin (
    id SERIAL PRIMARY KEY,
    code TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    supply_total INTEGER NOT NULL DEFAULT 0,
    supply_minted_total INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ
);

-- Index for quick lookup by code
CREATE INDEX IF NOT EXISTS idx_coin_code ON coin(code);
