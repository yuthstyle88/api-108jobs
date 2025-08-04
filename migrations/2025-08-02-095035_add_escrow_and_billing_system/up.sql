-- Add escrow balance to wallet table
ALTER TABLE wallet ADD COLUMN escrow_balance FLOAT8 DEFAULT 0.0;

-- Create billing status enum
CREATE TYPE billing_status AS ENUM (
    'QuotationPending',
    'OrderApproved',
    'PaidEscrow', 
    'WorkSubmitted',
    'RevisionRequested',
    'Completed',
    'Disputed',
    'Cancelled'
);

-- Create billing table for escrow invoices
CREATE TABLE billing (
    id SERIAL PRIMARY KEY,
    freelancer_id INTEGER NOT NULL REFERENCES local_user(id) ON UPDATE CASCADE ON DELETE CASCADE,
    employer_id INTEGER NOT NULL REFERENCES local_user(id) ON UPDATE CASCADE ON DELETE CASCADE,
    post_id INTEGER NOT NULL REFERENCES post(id) ON UPDATE CASCADE ON DELETE CASCADE,
    comment_id INTEGER REFERENCES comment(id) ON UPDATE CASCADE ON DELETE SET NULL,
    amount FLOAT8 NOT NULL CHECK (amount > 0),
    description TEXT NOT NULL,
    max_revisions INTEGER NOT NULL DEFAULT 0,
    revisions_used INTEGER NOT NULL DEFAULT 0,
    status billing_status NOT NULL DEFAULT 'QuotationPending',
    work_description TEXT,
    deliverable_url TEXT,
    revision_feedback TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ,
    paid_at TIMESTAMPTZ
);

-- Create indexes for billing table
CREATE INDEX idx_billing_freelancer_id ON billing(freelancer_id);
CREATE INDEX idx_billing_employer_id ON billing(employer_id);
CREATE INDEX idx_billing_post_id ON billing(post_id);
CREATE INDEX idx_billing_status ON billing(status);
CREATE INDEX idx_billing_created_at ON billing(created_at DESC);