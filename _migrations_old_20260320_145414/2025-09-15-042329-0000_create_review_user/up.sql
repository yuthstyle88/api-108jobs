CREATE TABLE IF NOT EXISTS user_review (
                                           id SERIAL PRIMARY KEY,
                                           reviewer_id INT NOT NULL REFERENCES person(id) ON UPDATE CASCADE ON DELETE CASCADE,
    reviewee_id INT NOT NULL REFERENCES person(id) ON UPDATE CASCADE ON DELETE CASCADE,
    workflow_id INT NOT NULL REFERENCES workflow(id) ON UPDATE CASCADE ON DELETE CASCADE,
    rating SMALLINT NOT NULL CHECK (rating >= 1 AND rating <= 5),
    comment TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NULL,
    CONSTRAINT uq_user_review_per_workflow UNIQUE (reviewer_id, reviewee_id, workflow_id)
    );

CREATE INDEX IF NOT EXISTS idx_user_review_reviewee_id ON user_review(reviewee_id);
CREATE INDEX IF NOT EXISTS idx_user_review_workflow_id ON user_review(workflow_id);
