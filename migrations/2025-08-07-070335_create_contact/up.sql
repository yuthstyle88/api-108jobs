-- Your SQL goes here
CREATE TABLE contact (
                         id SERIAL PRIMARY KEY,
                         local_user_id INTEGER NOT NULL REFERENCES local_user(id) ON DELETE CASCADE,
                         phone TEXT,
                         email TEXT,
                         secondary_email TEXT,
                         line_id TEXT,
                         facebook TEXT,
                         created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                         updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);