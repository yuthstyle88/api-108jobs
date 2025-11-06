CREATE TYPE topup_status AS ENUM ('Pending', 'Success', 'Expired');

CREATE TABLE wallet_topups
(
    id                 SERIAL PRIMARY KEY,

    -- Relationships
    local_user_id      int              NOT NULL REFERENCES local_user (id) ON DELETE CASCADE,

    -- Transaction details
    amount             DOUBLE PRECISION NOT NULL CHECK (amount > 0),
    currency_name      TEXT             NOT NULL,
    qr_id              TEXT             NOT NULL UNIQUE,
    cs_ext_expiry_time TIMESTAMPTZ      NOT NULL,
    status             topup_status     NOT NULL DEFAULT 'Pending',
    transferred        boolean          NOT NULL DEFAULT false,
    -- Metadata
    created_at         TIMESTAMPTZ      NOT NULL DEFAULT now(),
    updated_at         TIMESTAMPTZ      NOT NULL DEFAULT now(),
    paid_at            TIMESTAMPTZ, -- when confirmed success

    -- Idempotency & concurrency safety
    CONSTRAINT wallet_topups_unique_pair UNIQUE (local_user_id, qr_id)
);

-- Performance indexes
CREATE INDEX idx_wallet_topups_user_id ON wallet_topups (local_user_id);
CREATE INDEX idx_wallet_topups_status ON wallet_topups (status);
CREATE INDEX idx_wallet_topups_created_at ON wallet_topups (created_at DESC);
