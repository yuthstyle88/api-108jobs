CREATE TABLE proposals
(
    id           SERIAL PRIMARY KEY,
    description  TEXT             NOT NULL,
    budget       DOUBLE PRECISION NOT NULL CHECK (budget >= 0),
    working_days INTEGER          NOT NULL CHECK (working_days > 0),
    brief_url    VARCHAR(2048),
    user_id      INTEGER          NOT NULL REFERENCES local_user (id) ON DELETE CASCADE ON UPDATE CASCADE,
    post_id      INTEGER          NOT NULL REFERENCES post (id) ON DELETE CASCADE ON UPDATE CASCADE,
    community_id INTEGER          NOT NULL REFERENCES community (id) ON DELETE CASCADE ON UPDATE CASCADE,
    deleted_at   TIMESTAMP,
    created_at   TIMESTAMP        NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMP        NOT NULL DEFAULT NOW()
);

CREATE OR REPLACE FUNCTION update_updated_at()
    RETURNS TRIGGER AS
$$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trig_update_proposals_updated_at
    BEFORE UPDATE
    ON proposals
    FOR EACH ROW
EXECUTE PROCEDURE update_updated_at();

CREATE UNIQUE INDEX idx_proposals_user_job_unique ON proposals (user_id, post_id) WHERE deleted_at IS NULL;

CREATE INDEX idx_proposals_post_id ON proposals (post_id);

CREATE INDEX idx_proposals_service_id ON proposals (community_id);