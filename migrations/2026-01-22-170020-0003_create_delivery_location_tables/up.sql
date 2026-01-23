-- Current rider location per delivery
CREATE TABLE IF NOT EXISTS delivery_location_current (
  post_id     INT PRIMARY KEY REFERENCES post(id) ON DELETE CASCADE,
  rider_id    INT NOT NULL REFERENCES rider(id) ON DELETE CASCADE,
  lat         DOUBLE PRECISION NOT NULL,
  lng         DOUBLE PRECISION NOT NULL,
  heading     DOUBLE PRECISION,
  speed_kmh   DOUBLE PRECISION,
  accuracy_m  DOUBLE PRECISION,
  updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- History of rider locations (sampled)
CREATE TABLE IF NOT EXISTS delivery_location_history (
  id          BIGSERIAL PRIMARY KEY,
  post_id     INT NOT NULL REFERENCES post(id) ON DELETE CASCADE,
  rider_id    INT NOT NULL REFERENCES rider(id) ON DELETE CASCADE,
  lat         DOUBLE PRECISION NOT NULL,
  lng         DOUBLE PRECISION NOT NULL,
  heading     DOUBLE PRECISION,
  speed_kmh   DOUBLE PRECISION,
  accuracy_m  DOUBLE PRECISION,
  recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_delivery_location_history_post_time
  ON delivery_location_history (post_id, recorded_at DESC);
