-- Create comprehensive billing/invoice system with PersonId support
-- This migration creates the complete billing system from scratch

-- Create billing status enum with all required statuses
CREATE TYPE billing_status AS ENUM (
    'QuotationPending',     -- Freelancer sent quotation, waiting for employer approval
    'OrderApproved',        -- Employer approved quotation (deprecated, use PaidEscrow)
    'PaidEscrow',          -- Employer paid, money is in escrow
    'WorkSubmitted',       -- Freelancer submitted completed work
    'RevisionRequested',   -- Employer requested revision (deprecated)
    'RequestChange',       -- Employer requested changes to submitted work
    'Updated',             -- Freelancer updated work after revision request
    'Completed',           -- Work approved and payment released to freelancer
    'Disputed',            -- Dispute raised, requires resolution
    'Cancelled'            -- Order cancelled before completion
);

-- Create billing table for invoice/escrow management
CREATE TABLE billing (
    id SERIAL PRIMARY KEY,
    
    -- PersonId references for federation support
    freelancer_person_id INTEGER NOT NULL REFERENCES person(id) ON DELETE CASCADE,
    employer_person_id INTEGER NOT NULL REFERENCES person(id) ON DELETE CASCADE,
    
    -- Job references
    post_id INTEGER NOT NULL REFERENCES post(id) ON DELETE CASCADE,
    comment_id INTEGER REFERENCES comment(id) ON DELETE SET NULL,
    
    -- Financial details
    amount FLOAT8 NOT NULL CHECK (amount > 0),
    description TEXT NOT NULL,
    
    -- Revision management
    max_revisions INTEGER NOT NULL DEFAULT 0,
    revisions_used INTEGER NOT NULL DEFAULT 0,
    
    -- Status and workflow
    status billing_status NOT NULL DEFAULT 'QuotationPending',
    
    -- Work submission details
    work_description TEXT,
    deliverable_url TEXT,
    revision_feedback TEXT,
    
    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ,
    paid_at TIMESTAMPTZ
);

-- Create indexes for better performance
CREATE INDEX idx_billing_freelancer_person_id ON billing(freelancer_person_id);
CREATE INDEX idx_billing_employer_person_id ON billing(employer_person_id);
CREATE INDEX idx_billing_post_id ON billing(post_id);
CREATE INDEX idx_billing_comment_id ON billing(comment_id);
CREATE INDEX idx_billing_status ON billing(status);
CREATE INDEX idx_billing_created_at ON billing(created_at DESC);
CREATE INDEX idx_billing_amount ON billing(amount);

-- Add constraints for data integrity
ALTER TABLE billing ADD CONSTRAINT chk_billing_revisions_non_negative 
    CHECK (revisions_used >= 0 AND max_revisions >= 0);
ALTER TABLE billing ADD CONSTRAINT chk_billing_revisions_within_limit 
    CHECK (revisions_used <= max_revisions);