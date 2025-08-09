-- Create education table linked directly to person
CREATE TABLE education (
    id SERIAL PRIMARY KEY,
    person_id INTEGER NOT NULL REFERENCES person(id) ON DELETE CASCADE,
    school_name TEXT NOT NULL,
    major TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create work_experience table linked directly to person
CREATE TABLE work_experience (
    id SERIAL PRIMARY KEY,
    person_id INTEGER NOT NULL REFERENCES person(id) ON DELETE CASCADE,
    company_name TEXT NOT NULL,
    position TEXT NOT NULL,
    start_month TEXT,
    start_year INTEGER,
    end_month TEXT,
    end_year INTEGER,
    is_current BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create skills table linked directly to person
CREATE TABLE skills (
    id SERIAL PRIMARY KEY,
    person_id INTEGER NOT NULL REFERENCES person(id) ON DELETE CASCADE,
    skill_name TEXT NOT NULL,
    level_id INTEGER, -- Can reference a skill_levels table if needed
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create certificates table linked directly to person
CREATE TABLE certificates (
    id SERIAL PRIMARY KEY,
    person_id INTEGER NOT NULL REFERENCES person(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create indexes for better performance
CREATE INDEX idx_education_person_id ON education(person_id);
CREATE INDEX idx_work_experience_person_id ON work_experience(person_id);
CREATE INDEX idx_skills_person_id ON skills(person_id);
CREATE INDEX idx_certificates_person_id ON certificates(person_id);
