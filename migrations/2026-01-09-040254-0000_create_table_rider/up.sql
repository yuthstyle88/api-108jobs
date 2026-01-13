-- Enums
CREATE TYPE vehicle_type AS ENUM (
    'Motorcycle',
    'Bicycle',
    'Car'
);

CREATE TYPE rider_verification_status AS ENUM (
    'Pending',
    'Verified',
    'Rejected'
);

-- Table
CREATE TABLE rider
(
    id                   SERIAL PRIMARY KEY,

    -- References
    user_id              INT                       NOT NULL
        REFERENCES local_user (id)
            ON UPDATE CASCADE ON DELETE CASCADE,

    person_id            INT                       NOT NULL
        REFERENCES person (id)
            ON UPDATE CASCADE ON DELETE CASCADE,

    -- Vehicle
    vehicle_type         vehicle_type              NOT NULL,
    vehicle_plate_number VARCHAR,
    license_number       VARCHAR,
    license_expiry_date  TIMESTAMPTZ,

    -- Verification
    is_verified          BOOLEAN                   NOT NULL DEFAULT FALSE,
    is_active            BOOLEAN                   NOT NULL DEFAULT TRUE,
    verification_status  rider_verification_status NOT NULL DEFAULT 'Pending',

    -- Performance
    rating               DOUBLE PRECISION          NOT NULL DEFAULT 0,
    completed_jobs       INT                       NOT NULL DEFAULT 0,
    total_jobs           INT                       NOT NULL DEFAULT 0,
    total_earnings       DOUBLE PRECISION          NOT NULL DEFAULT 0,
    pending_earnings     DOUBLE PRECISION          NOT NULL DEFAULT 0,

    -- Availability
    is_online            BOOLEAN                   NOT NULL DEFAULT FALSE,
    accepting_jobs       BOOLEAN                   NOT NULL DEFAULT TRUE,

    -- Timestamps
    joined_at            TIMESTAMPTZ,
    last_active_at       TIMESTAMPTZ,
    verified_at          TIMESTAMPTZ
);

-- Helpful indexes
CREATE INDEX idx_rider_user_id ON rider (user_id);
CREATE INDEX idx_rider_person_id ON rider (person_id);
CREATE INDEX idx_rider_is_online ON rider (is_online);
CREATE INDEX idx_rider_accepting_jobs ON rider (accepting_jobs);
