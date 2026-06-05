-- Phase 2: wallet versioning + escrow hold ledger.
--
-- Concurrency strategy: SELECT FOR UPDATE inside run_transaction is the primary
-- mechanism (already in use in WalletModel::apply_op_on). The `version` column
-- below is added per spec for:
--   * defense-in-depth for any code path that bypasses apply_op_on
--   * future explicit CAS / external auditing
-- It is incremented in lockstep with every balance mutation.
--
-- Adds:
--   wallet.version           BIGINT NOT NULL DEFAULT 0
--   wallet_hold              per-billing escrow ledger
--
-- The ledger lets us:
--   * derive balance_outstanding as SUM(amount) WHERE wallet_id=? AND status='Active'
--   * reject duplicate approve calls via a partial unique index on (billing_id) WHERE Active
--   * make release/refund idempotent at the DB level

ALTER TABLE wallet
  ADD COLUMN version BIGINT NOT NULL DEFAULT 0;

-- Backfill is implicit: DEFAULT 0 already covers existing rows.

CREATE TABLE wallet_hold (
  id              BIGSERIAL PRIMARY KEY,
  wallet_id       INT NOT NULL REFERENCES wallet(id)  ON DELETE CASCADE,
  billing_id      INT NOT NULL REFERENCES billing(id) ON DELETE CASCADE,
  amount          INT NOT NULL CHECK (amount > 0),
  -- Status is text + CHECK rather than a PG ENUM so this migration stays
  -- trivially reversible (no enum drop hazards if a future migration extends it).
  status          TEXT NOT NULL CHECK (status IN ('Active', 'Released', 'Captured')),
  idempotency_key TEXT,
  created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
  released_at     TIMESTAMPTZ
);

CREATE INDEX idx_wallet_hold_wallet  ON wallet_hold(wallet_id);
CREATE INDEX idx_wallet_hold_billing ON wallet_hold(billing_id);
CREATE INDEX idx_wallet_hold_status  ON wallet_hold(status);

-- A given billing may have at most ONE active hold. A duplicate approve attempt
-- collides on insert and surfaces as DuplicateWalletHold to the application.
CREATE UNIQUE INDEX uq_wallet_hold_active_per_billing
  ON wallet_hold(billing_id)
  WHERE status = 'Active';

-- Optional global idempotency key for retried workflow operations.
CREATE UNIQUE INDEX uq_wallet_hold_idem
  ON wallet_hold(idempotency_key)
  WHERE idempotency_key IS NOT NULL;
