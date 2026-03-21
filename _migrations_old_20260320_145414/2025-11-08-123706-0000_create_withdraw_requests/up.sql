-- 1. Create enum type for status
CREATE TYPE withdraw_status AS ENUM ('Pending', 'Rejected', 'Completed');

-- 2. Create table withdraw_requests
CREATE TABLE withdraw_requests
(
    id                   serial PRIMARY KEY,
    local_user_id        int             NOT NULL REFERENCES local_user (id) ON DELETE CASCADE,
    wallet_id            int             NOT NULL REFERENCES wallet (id) ON DELETE CASCADE,
    user_bank_account_id int             NOT NULL REFERENCES user_bank_accounts (id)
        ON DELETE RESTRICT,
    amount               int             NOT NULL DEFAULT 0,
    status               withdraw_status NOT NULL DEFAULT 'Pending',
    reason               text,
    created_at           timestamptz     NOT NULL DEFAULT now(),
    updated_at           timestamptz     NOT NULL DEFAULT now()
);
