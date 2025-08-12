-- Create language_profile table for person language proficiencies
CREATE TABLE language_profile (
    id SERIAL PRIMARY KEY,
    person_id INTEGER NOT NULL REFERENCES person(id) ON DELETE CASCADE,
    lang VARCHAR(100) NOT NULL,
    level_name VARCHAR(50) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ
);

-- Create indexes for performance
CREATE INDEX idx_language_profile_person_id ON language_profile(person_id);
CREATE INDEX idx_language_profile_lang ON language_profile(lang);

-- Ensure no duplicate language per person
CREATE UNIQUE INDEX idx_language_profile_person_lang ON language_profile(person_id, lang);