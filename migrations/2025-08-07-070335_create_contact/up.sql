-- Your SQL goes here
CREATE TABLE contact (
                         id SERIAL PRIMARY KEY,
                         phone TEXT,
                         email TEXT,
                         secondary_email TEXT,
                         line_id TEXT,
                         facebook TEXT,
                         created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                         updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);