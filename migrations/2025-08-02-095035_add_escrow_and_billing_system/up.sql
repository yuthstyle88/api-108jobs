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
    freelancer_id INT NOT NULL REFERENCES local_user(id) ON UPDATE CASCADE ON DELETE CASCADE,
    employer_id   INT NOT NULL REFERENCES local_user(id) ON UPDATE CASCADE ON DELETE CASCADE,
    post_id       INT NOT NULL REFERENCES post(id)        ON UPDATE CASCADE ON DELETE CASCADE,
    comment_id    INT REFERENCES comment(id)              ON UPDATE CASCADE ON DELETE SET NULL,
    amount        INT NOT NULL CHECK (amount > 0),
    description   TEXT NOT NULL,
    status        billing_status NOT NULL DEFAULT 'QuotationPending',
    work_description TEXT,
    deliverable_url TEXT,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at    TIMESTAMPTZ,
    paid_at       TIMESTAMPTZ
);

-- Create indexes for billing table
CREATE INDEX idx_billing_freelancer_id ON billing(freelancer_id);
CREATE INDEX idx_billing_employer_id  ON billing(employer_id);
CREATE INDEX idx_billing_post_id      ON billing(post_id);
CREATE INDEX idx_billing_status       ON billing(status);
CREATE INDEX idx_billing_created_at   ON billing(created_at DESC);