ALTER TABLE person
ADD COLUMN work_samples JSONB,
ADD COLUMN portfolio_pics JSONB;

CREATE INDEX idx_person_portfolio_pics ON person USING GIN (portfolio_pics);
CREATE INDEX idx_person_work_samples ON person USING GIN (work_samples);