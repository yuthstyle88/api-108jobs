ALTER TABLE email_verification
    RENAME COLUMN verification_code TO verification_token;

ALTER TABLE local_site
DROP COLUMN verify_with_otp;
