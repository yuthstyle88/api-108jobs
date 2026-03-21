ALTER TABLE local_site
    ADD COLUMN verify_with_otp boolean NOT NULL DEFAULT true;
ALTER TABLE email_verification
    RENAME COLUMN verification_token TO verification_code;