-- Create comprehensive wallet system with PersonId support and BigDecimal precision
-- This migration creates the complete wallet system from scratch

-- Create enhanced wallet table with all features
CREATE TABLE wallet (
    id SERIAL PRIMARY KEY,
    
    -- Balance fields using DECIMAL for precise financial calculations
    available_balance DECIMAL(15,2) DEFAULT 0.00 NOT NULL,
    escrow_balance DECIMAL(15,2) DEFAULT 0.00 NOT NULL,
    pending_in DECIMAL(15,2) DEFAULT 0.00 NOT NULL,
    pending_out DECIMAL(15,2) DEFAULT 0.00 NOT NULL,
    reserved_balance DECIMAL(15,2) DEFAULT 0.00 NOT NULL,
    
    -- Financial controls and security
    is_frozen BOOLEAN DEFAULT FALSE NOT NULL,
    freeze_reason VARCHAR(500),
    currency VARCHAR(3) DEFAULT 'USD' NOT NULL,
    version INTEGER DEFAULT 1 NOT NULL, -- For optimistic locking
    
    -- PersonId for federation support (directly linked to person)
    person_id INTEGER NOT NULL REFERENCES person(id) ON DELETE CASCADE,
    updated_by_person_id INTEGER REFERENCES person(id) ON DELETE SET NULL,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ,
    last_transaction_at TIMESTAMPTZ
);

-- Add constraints to prevent negative balances
ALTER TABLE wallet ADD CONSTRAINT chk_available_balance_non_negative 
    CHECK (available_balance >= 0);
ALTER TABLE wallet ADD CONSTRAINT chk_escrow_balance_non_negative 
    CHECK (escrow_balance >= 0);
ALTER TABLE wallet ADD CONSTRAINT chk_pending_in_non_negative 
    CHECK (pending_in >= 0);
ALTER TABLE wallet ADD CONSTRAINT chk_pending_out_non_negative 
    CHECK (pending_out >= 0);
ALTER TABLE wallet ADD CONSTRAINT chk_reserved_balance_non_negative 
    CHECK (reserved_balance >= 0);

-- Add constraint for valid currency codes
ALTER TABLE wallet ADD CONSTRAINT chk_valid_currency 
    CHECK (currency IN ('USD', 'THB', 'VND', 'EUR', 'SGD'));

-- Add unique constraint to ensure one wallet per person
ALTER TABLE wallet ADD CONSTRAINT wallet_person_id_unique UNIQUE (person_id);

-- Create indexes for better performance
CREATE INDEX idx_wallet_person_id ON wallet(person_id);
CREATE INDEX idx_wallet_updated_by_person_id ON wallet(updated_by_person_id);
CREATE INDEX idx_wallet_currency ON wallet(currency);
CREATE INDEX idx_wallet_is_frozen ON wallet(is_frozen);
CREATE INDEX idx_wallet_last_transaction ON wallet(last_transaction_at);

-- Add trigger to automatically update version for optimistic locking
CREATE OR REPLACE FUNCTION increment_wallet_version()
RETURNS TRIGGER AS $$
BEGIN
    NEW.version = OLD.version + 1;
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_wallet_version_update
    BEFORE UPDATE ON wallet
    FOR EACH ROW
    EXECUTE FUNCTION increment_wallet_version();

-- Keep wallet_id in local_user for backward compatibility (will be deprecated later)
ALTER TABLE local_user ADD COLUMN wallet_id INTEGER REFERENCES wallet(id) ON UPDATE CASCADE ON DELETE SET NULL;
CREATE INDEX idx_local_user_wallet_id ON local_user(wallet_id);