-- Table for tracking employer ratings of riders for completed deliveries
CREATE TABLE IF NOT EXISTS delivery_rider_rating (
    id SERIAL PRIMARY KEY,
    post_id INT NOT NULL REFERENCES post(id) ON UPDATE CASCADE ON DELETE CASCADE,
    employer_id INT NOT NULL REFERENCES person(id) ON UPDATE CASCADE ON DELETE CASCADE,
    rider_id INT NOT NULL REFERENCES rider(id) ON UPDATE CASCADE ON DELETE CASCADE,
    rating SMALLINT NOT NULL CHECK (rating >= 1 AND rating <= 5),
    comment TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ,
    CONSTRAINT uq_delivery_rider_rating UNIQUE (post_id, employer_id, rider_id)
);

-- Indexes for common queries
CREATE INDEX idx_delivery_rider_rating_rider_id ON delivery_rider_rating(rider_id);
CREATE INDEX idx_delivery_rider_rating_post_id ON delivery_rider_rating(post_id);
CREATE INDEX idx_delivery_rider_rating_employer_id ON delivery_rider_rating(employer_id);
