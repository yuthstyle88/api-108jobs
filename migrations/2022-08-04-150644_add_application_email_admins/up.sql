-- Adding a field to multilang admins for new applications
ALTER TABLE site
    ADD COLUMN application_email_admins boolean NOT NULL DEFAULT FALSE;

