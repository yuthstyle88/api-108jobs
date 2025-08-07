-- Add country field to person table for regional features
ALTER TABLE person
    ADD COLUMN country VARCHAR(100);

-- Create index for better query performance
CREATE INDEX idx_person_country ON person(country);