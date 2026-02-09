-- Multi-currency support for taxi rider system
-- Admin can manage currencies and conversion rates

-- Add RiderConfirmed status to DeliveryStatus enum (for taxi rides)
ALTER TYPE delivery_status ADD VALUE IF NOT EXISTS 'RiderConfirmed';

-- Create payment_method enum for rides (taxi and cargo)
CREATE TYPE payment_method AS ENUM ('cash', 'coin');

-- Currency table with admin-managed rates
CREATE TABLE currency (
    id SERIAL PRIMARY KEY,
    code VARCHAR(3) NOT NULL UNIQUE,
    name VARCHAR(50) NOT NULL,
    symbol VARCHAR(10) NOT NULL,

    -- CONVERSION RATE (Admin Managed!)
    -- How many units of this currency = 1 Coin
    -- Examples:
    --   THB: 1 Coin = 0.01 THB (1 satang), rate = 1
    --        So 100 Coins = 1 THB (display: 1.00)
    --   IDR: 1 Coin = 100 Rupiah, rate = 100
    --        So 5000 Coins = 500,000 Rupiah (display: 500.000)
    --   VND: 1 Coin = 100 Dong, rate = 100
    --        So 5000 Coins = 500,000 Dong (display: 500.000)
    coin_to_currency_rate INTEGER NOT NULL DEFAULT 1,

    -- Display formatting
    decimal_places INTEGER NOT NULL DEFAULT 2,
    thousands_separator VARCHAR(1) DEFAULT ',',
    decimal_separator VARCHAR(1) DEFAULT '.',
    symbol_position VARCHAR(10) DEFAULT 'prefix',

    -- Status
    is_active BOOLEAN DEFAULT TRUE,
    is_default BOOLEAN DEFAULT FALSE,

    -- Admin tracking
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ,
    rate_last_updated_at TIMESTAMPTZ,
    rate_last_updated_by INTEGER REFERENCES local_user(id)
);

-- Rate history audit log
CREATE TABLE currency_rate_history (
    id SERIAL PRIMARY KEY,
    currency_id INTEGER NOT NULL REFERENCES currency(id) ON DELETE CASCADE,
    old_rate INTEGER NOT NULL,
    new_rate INTEGER NOT NULL,
    changed_by INTEGER REFERENCES local_user(id),
    changed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    reason TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_currency_rate_history_currency_date ON currency_rate_history(currency_id, changed_at DESC);

-- Pricing configuration per currency (admin manages)
CREATE TABLE pricing_config (
    id SERIAL PRIMARY KEY,
    currency_id INTEGER NOT NULL REFERENCES currency(id),
    name VARCHAR(100) NOT NULL,

    -- Base pricing (stored in Coins internally)
    base_fare_coin INTEGER NOT NULL,
    time_charge_per_minute_coin INTEGER NOT NULL,
    minimum_charge_minutes INTEGER DEFAULT 10,
    distance_charge_per_km_coin INTEGER NOT NULL,

    -- Payment options
    accepts_cash BOOLEAN DEFAULT TRUE,
    accepts_coin BOOLEAN DEFAULT TRUE,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ
);

CREATE INDEX idx_pricing_config_currency_active ON pricing_config(currency_id, is_active);

-- Ride sessions for taxi-style rides with dynamic pricing
CREATE TABLE ride_session (
    id SERIAL PRIMARY KEY,
    post_id INTEGER NOT NULL REFERENCES post(id) ON DELETE CASCADE,
    rider_id INTEGER REFERENCES rider(id) ON DELETE SET NULL,  -- NULL until rider accepts
    employer_id INTEGER NOT NULL REFERENCES local_user(id),

    -- Pricing config used
    pricing_config_id INTEGER REFERENCES pricing_config(id),

    -- Route & Payment
    pickup_address TEXT NOT NULL,
    pickup_lat FLOAT8,
    pickup_lng FLOAT8,
    dropoff_address TEXT NOT NULL,
    dropoff_lat FLOAT8,
    dropoff_lng FLOAT8,
    pickup_note TEXT,

    -- Payment method
    payment_method payment_method NOT NULL,  -- Uses enum: 'cash' or 'coin'
    payment_status VARCHAR(20) DEFAULT 'pending',

    -- Session state - uses DeliveryStatus enum (shared with cargo delivery)
    status delivery_status NOT NULL DEFAULT 'Pending',
    -- Taxi flow: Pending -> Assigned -> RiderConfirmed -> EnRouteToPickup -> PickedUp -> EnRouteToDropoff -> Delivered
    -- Cargo flow: Pending -> Assigned -> EnRouteToPickup -> PickedUp -> EnRouteToDropoff -> Delivered

    -- Timestamps
    requested_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    rider_assigned_at TIMESTAMPTZ,
    rider_confirmed_at TIMESTAMPTZ,
    arrived_at_pickup_at TIMESTAMPTZ,
    ride_started_at TIMESTAMPTZ,
    ride_completed_at TIMESTAMPTZ,

    -- Real-time meter data (stored in Coins)
    current_price_coin INTEGER DEFAULT 0,

    -- Final calculated values (set on completion)
    total_distance_km FLOAT8,
    total_duration_minutes INTEGER,
    final_price_coin INTEGER,

    -- Pricing breakdown (stored in Coins)
    base_fare_applied_coin INTEGER,
    time_charge_applied_coin INTEGER,
    distance_charge_applied_coin INTEGER,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ
);

CREATE INDEX idx_ride_session_post ON ride_session(post_id);
CREATE INDEX idx_ride_session_rider ON ride_session(rider_id);
CREATE INDEX idx_ride_session_status ON ride_session(status);

-- Real-time ride meter snapshots (for WebSocket updates & audit)
CREATE TABLE ride_meter_snapshot (
    id SERIAL PRIMARY KEY,
    ride_session_id INTEGER NOT NULL REFERENCES ride_session(id) ON DELETE CASCADE,

    -- Snapshot data
    elapsed_minutes INTEGER NOT NULL,
    distance_km FLOAT8 NOT NULL,
    current_price_coin INTEGER NOT NULL,

    -- Breakdown (in Coins)
    base_fare_coin INTEGER NOT NULL,
    time_charge_coin INTEGER NOT NULL,
    distance_charge_coin INTEGER NOT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_ride_meter_snapshot_session ON ride_meter_snapshot(ride_session_id, created_at);

-- Enable auto-update for updated_at columns
SELECT diesel_manage_updated_at('currency');
SELECT diesel_manage_updated_at('pricing_config');
SELECT diesel_manage_updated_at('ride_session');

-- Seed default currency (THB)
INSERT INTO currency (code, name, symbol, coin_to_currency_rate, decimal_places, thousands_separator, decimal_separator, symbol_position, is_default)
VALUES ('THB', 'Thai Baht', '฿', 1, 2, ',', '.', 'prefix', TRUE);

-- Seed default pricing for THB (50/10/10 stored as Coins)
-- Base: 50 THB = 5000 Coins
-- Time: 1 THB/min = 100 Coins/min (charged every 10 min = 1000 Coins)
-- Distance: 10 THB/km = 1000 Coins/km
INSERT INTO pricing_config (currency_id, name, base_fare_coin, time_charge_per_minute_coin, minimum_charge_minutes, distance_charge_per_km_coin)
SELECT id, 'Standard Taxi', 5000, 100, 10, 1000
FROM currency WHERE code = 'THB';
