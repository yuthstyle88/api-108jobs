-- Delivery details table, 1-1 with post
CREATE TABLE IF NOT EXISTS delivery_details (
  id                   SERIAL PRIMARY KEY,
  post_id              INT NOT NULL UNIQUE
    REFERENCES post (id)
      ON UPDATE CASCADE ON DELETE CASCADE,

  -- Locations
  pickup_address       TEXT NOT NULL,
  pickup_lat           DOUBLE PRECISION,
  pickup_lng           DOUBLE PRECISION,
  dropoff_address      TEXT NOT NULL,
  dropoff_lat          DOUBLE PRECISION,
  dropoff_lng          DOUBLE PRECISION,

  -- Package
  package_description  TEXT,
  package_weight_kg    DOUBLE PRECISION,
  package_size         VARCHAR,
  fragile              BOOLEAN NOT NULL DEFAULT FALSE,
  requires_signature   BOOLEAN NOT NULL DEFAULT FALSE,

  -- Constraints
  vehicle_required     vehicle_type,
  latest_pickup_at     TIMESTAMPTZ,
  latest_dropoff_at    TIMESTAMPTZ,

  -- Contacts
  sender_name          VARCHAR,
  sender_phone         VARCHAR,
  receiver_name        VARCHAR,
  receiver_phone       VARCHAR,

  -- Payment options
  cash_on_delivery     BOOLEAN NOT NULL DEFAULT FALSE,
  cod_amount           DOUBLE PRECISION,

  -- Tracking state
  status               delivery_status NOT NULL DEFAULT 'Pending',

  -- Audit
  created_at           TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  updated_at           TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_delivery_details_post_id ON delivery_details (post_id);
CREATE INDEX IF NOT EXISTS idx_delivery_details_status ON delivery_details (status);
