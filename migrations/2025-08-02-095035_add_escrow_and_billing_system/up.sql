-- Add escrow balance to wallet table (use NUMERIC for money)
ALTER TABLE wallet ADD COLUMN escrow_balance NUMERIC(12,2) NOT NULL DEFAULT 0.00;

-- Create billing status enum (finalized states)
CREATE TYPE billing_status AS ENUM (
    'QuotationPending',
    'OrderApproved',
    'PaidEscrow',
    'WorkSubmitted',
    'Completed',
    'Cancelled'
);

-- Create billing table for escrow jobs (money in NUMERIC)
CREATE TABLE billing (
    id SERIAL PRIMARY KEY,
    freelancer_id INTEGER NOT NULL REFERENCES local_user(id) ON UPDATE CASCADE ON DELETE CASCADE,
    employer_id   INTEGER NOT NULL REFERENCES local_user(id) ON UPDATE CASCADE ON DELETE CASCADE,
    post_id       INTEGER NOT NULL REFERENCES post(id)        ON UPDATE CASCADE ON DELETE CASCADE,
    comment_id    INTEGER REFERENCES comment(id)              ON UPDATE CASCADE ON DELETE SET NULL,

    amount        NUMERIC(12,2) NOT NULL CHECK (amount > 0),
    description   TEXT NOT NULL,

    status        billing_status NOT NULL DEFAULT 'QuotationPending',
    work_description TEXT,
    deliverable_url TEXT,

    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ,
    paid_at       TIMESTAMPTZ
);

-- Keep updated_at fresh via common trigger
CREATE TRIGGER set_updated_at_billing
    BEFORE UPDATE ON billing
    FOR EACH ROW
    EXECUTE FUNCTION r.set_updated_at();

-- Create indexes for billing table
CREATE INDEX idx_billing_freelancer_id ON billing(freelancer_id);
CREATE INDEX idx_billing_employer_id  ON billing(employer_id);
CREATE INDEX idx_billing_post_id      ON billing(post_id);
CREATE INDEX idx_billing_status       ON billing(status);
CREATE INDEX idx_billing_created_at   ON billing(created_at DESC);