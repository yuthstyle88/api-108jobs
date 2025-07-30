-- Tạo bảng job_posts
CREATE TABLE job_posts
(
    id                  SERIAL PRIMARY KEY,
    job_title           VARCHAR(200)                                                           NOT NULL,
    description         VARCHAR(2000)                                                          NOT NULL,
    is_english_required BOOLEAN     DEFAULT FALSE                                              NOT NULL,
    example_url         VARCHAR(2048),
    budget              DOUBLE PRECISION                                                       NOT NULL CHECK (budget > 0),
    deadline            DATE,
    is_anonymous_post   BOOLEAN     DEFAULT FALSE                                              NOT NULL,
    creator_id          INTEGER REFERENCES local_user (id) ON UPDATE CASCADE ON DELETE CASCADE NOT NULL,
    service_catalog_id  INTEGER REFERENCES community (id) ON UPDATE CASCADE ON DELETE CASCADE  NOT NULL,
    working_from        working_from_enum                                                      NOT NULL,
    intended_use        intended_use_enum                                                      NOT NULL,
    created_at          TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at          TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE OR REPLACE FUNCTION update_updated_at()
    RETURNS TRIGGER AS
$$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trig_update_job_posts_updated_at
    BEFORE UPDATE
    ON job_posts
    FOR EACH ROW
EXECUTE PROCEDURE update_updated_at();

CREATE INDEX idx_job_posts_creator_id ON job_posts (creator_id);
CREATE INDEX idx_job_posts_service_catalog_id ON job_posts (service_catalog_id);
CREATE INDEX idx_job_posts_working_from ON job_posts (working_from);