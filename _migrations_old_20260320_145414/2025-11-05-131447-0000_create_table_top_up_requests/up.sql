CREATE TYPE top_up_status AS ENUM ('Pending', 'Success', 'Expired');

CREATE TABLE top_up_requests
(
    id                 SERIAL PRIMARY KEY,

    -- Relationships
    local_user_id      int              NOT NULL REFERENCES local_user (id) ON DELETE CASCADE,

    -- Transaction details
    amount             DOUBLE PRECISION NOT NULL CHECK (amount > 0),
    currency_name      TEXT             NOT NULL,
    qr_id              TEXT             NOT NULL UNIQUE,
    cs_ext_expiry_time TIMESTAMPTZ      NOT NULL,
    status             top_up_status    NOT NULL DEFAULT 'Pending',
    transferred        boolean          NOT NULL DEFAULT false,
    -- Metadata
    created_at         TIMESTAMPTZ      NOT NULL DEFAULT now(),
    updated_at         TIMESTAMPTZ      NOT NULL DEFAULT now(),
    paid_at            TIMESTAMPTZ, -- when confirmed success

    -- Idempotency & concurrency safety
    CONSTRAINT top_ups_unique_pair UNIQUE (local_user_id, qr_id)
);

-- Performance indexes
CREATE INDEX idx_top_up_requests_user_id ON top_up_requests (local_user_id);
CREATE INDEX idx_top_up_requests_status ON top_up_requests (status);
CREATE INDEX idx_top_up_requests_created_at ON top_up_requests (created_at DESC);
