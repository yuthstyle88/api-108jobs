ALTER TABLE education
    ADD CONSTRAINT education_uq_person_school_major UNIQUE (person_id, school_name, major);

ALTER TABLE language_profile
    ADD CONSTRAINT language_profile_uq_person_lang UNIQUE (person_id, lang);

ALTER TABLE skills
    ADD CONSTRAINT skills_uq_person_skill UNIQUE (person_id, skill_name);

ALTER TABLE work_experience
    ADD CONSTRAINT work_experience_uq_person_company_position UNIQUE (person_id, company_name, position);

ALTER TABLE certificates
    ADD CONSTRAINT certificates_uq_person_name UNIQUE (person_id, name);
