-- Create comprehensive transaction logging system with PersonId support
-- This migration creates the complete transaction system from scratch

-- Create enum for transaction types
CREATE TYPE transaction_type_enum AS ENUM (
    'JobPayment',           -- Direct payment from employer to freelancer (disabled for security)
    'EscrowDeposit',        -- Money moved from wallet to escrow when employer approves quotation
    'EscrowRelease',        -- Money released from escrow to freelancer when work is approved
    'EscrowRefund',         -- Money refunded from escrow back to employer if cancelled
    'AdminTopUp',           -- Admin adds money to user wallet
    'AdminWithdraw',        -- Admin withdraws money from user wallet  
    'UserDeposit',          -- User deposits money into their wallet (disabled for security)
    'UserWithdraw',         -- User withdraws money from their wallet
    'SystemFee',            -- Platform fee deduction
    'Refund',               -- General refund transaction
    'Bonus',                -- Bonus payment
    'Penalty'               -- Penalty deduction
);

-- Create enum for transaction status
CREATE TYPE transaction_status_enum AS ENUM (
    'Pending',              -- Transaction initiated but not processed
    'Processing',           -- Transaction being processed
    'Completed',            -- Transaction successfully completed
    'Failed',               -- Transaction failed
    'Cancelled',            -- Transaction was cancelled
    'Refunded'              -- Transaction was refunded
);

-- Create transaction table for comprehensive audit logging
CREATE TABLE transaction (
    id SERIAL PRIMARY KEY,
    
    -- PersonId references for federation support (can be null for system transactions)
    from_user_id INTEGER REFERENCES person(id) ON DELETE SET NULL,
    to_user_id INTEGER REFERENCES person(id) ON DELETE SET NULL,
    
    -- Financial details with BigDecimal precision
    amount DECIMAL(15,2) NOT NULL CHECK (amount > 0),
    
    -- Transaction classification
    transaction_type transaction_type_enum NOT NULL,
    status transaction_status_enum NOT NULL DEFAULT 'Pending',
    
    -- Related records (optional)
    billing_id INTEGER REFERENCES billing(id) ON DELETE SET NULL,
    post_id INTEGER REFERENCES post(id) ON DELETE SET NULL,
    
    -- Transaction details
    description TEXT NOT NULL,
    reference_number VARCHAR(100),
    
    -- Extensible metadata as JSONB
    metadata JSONB,
    
    -- Timestamps for audit trail
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ
);

-- Create indexes for optimal performance
CREATE INDEX idx_transaction_from_user ON transaction(from_user_id);
CREATE INDEX idx_transaction_to_user ON transaction(to_user_id);
CREATE INDEX idx_transaction_billing ON transaction(billing_id);
CREATE INDEX idx_transaction_post ON transaction(post_id);
CREATE INDEX idx_transaction_type ON transaction(transaction_type);
CREATE INDEX idx_transaction_status ON transaction(status);
CREATE INDEX idx_transaction_created_at ON transaction(created_at DESC);
CREATE INDEX idx_transaction_amount ON transaction(amount);
CREATE INDEX idx_transaction_reference ON transaction(reference_number);

-- Add constraints for data integrity
ALTER TABLE transaction ADD CONSTRAINT chk_transaction_users_not_same 
    CHECK (from_user_id IS DISTINCT FROM to_user_id OR from_user_id IS NULL OR to_user_id IS NULL);