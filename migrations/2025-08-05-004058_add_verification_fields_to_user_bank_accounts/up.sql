-- Add verification fields to user_bank_accounts table
ALTER TABLE user_bank_accounts ADD COLUMN is_verified BOOLEAN DEFAULT FALSE NOT NULL;
ALTER TABLE user_bank_accounts ADD COLUMN verification_image_path VARCHAR(500);

-- Create index for efficient filtering by verification status
CREATE INDEX idx_user_bank_accounts_is_verified ON user_bank_accounts(is_verified);