-- Add verification fields to user_bank_accounts table
ALTER TABLE user_bank_accounts ADD COLUMN verification_image_path VARCHAR(500);
