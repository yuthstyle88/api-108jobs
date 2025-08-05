-- Remove verification fields from user_bank_accounts table
DROP INDEX IF EXISTS idx_user_bank_accounts_is_verified;
ALTER TABLE user_bank_accounts DROP COLUMN IF EXISTS verification_image_path;
ALTER TABLE user_bank_accounts DROP COLUMN IF EXISTS is_verified;
