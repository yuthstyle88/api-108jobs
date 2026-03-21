-- Seed platform assets: wallet and coin
-- This migration creates the singleton platform wallet and platform coin
-- using migration-based seeding (professional approach)

-- Add singleton constraint to ensure only one platform wallet exists
-- First, remove the old partial index if it exists
DROP INDEX IF EXISTS idx_wallet_platform;

-- Create a unique index where is_platform = TRUE (allows only one platform wallet)
CREATE UNIQUE INDEX idx_wallet_platform_singleton ON wallet (is_platform) WHERE is_platform = TRUE;

-- Seed platform wallet with zero balances (singleton)
-- Use DO block for idempotent insert since ON CONFLICT doesn't work with partial indexes
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM wallet WHERE is_platform = TRUE) THEN
        INSERT INTO wallet (balance_total, balance_available, balance_outstanding, is_platform, created_at, updated_at)
        VALUES (0, 0, 0, TRUE, NOW(), NOW());
    END IF;
END $$;

-- Seed platform coin with configurable initial supply
-- Code: 108JC (108Jobs Coin) - can be customized via environment
-- Initial supply: 1 billion coins (1,000,000,000 satang/coins)
-- This represents the platform's total money supply that will be distributed
INSERT INTO coin (code, name, supply_total, supply_minted_total, created_at, updated_at)
VALUES (
  '108JC',
  '108Jobs Coin',
  1000000000,  -- 1 billion in internal units (coins/satang)
  1000000000,  -- Initially fully minted
  NOW(),
  NOW()
)
ON CONFLICT (code) DO NOTHING;  -- Idempotent: skip if coin already exists

-- Add comment for documentation
COMMENT ON TABLE wallet IS 'User and platform wallets. Platform wallet (is_platform=TRUE) is a singleton used as the source of truth for all platform transactions. It can have negative balance for accounting purposes.';
COMMENT ON TABLE coin IS 'Platform coin definition. The 108JC coin is the internal currency unit used throughout the system.';
COMMENT ON INDEX idx_wallet_platform_singleton IS 'Ensures only one platform wallet exists (singleton pattern).';
