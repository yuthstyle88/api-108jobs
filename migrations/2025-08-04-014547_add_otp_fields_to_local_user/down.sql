-- Remove OTP fields from local_user table and additional fields from person table
ALTER TABLE local_user
   DROP COLUMN IF EXISTS otp_secret,
   DROP COLUMN IF EXISTS otp_url;

-- Remove additional fields from person table
ALTER TABLE person
   DROP COLUMN IF EXISTS wallet_id,
   DROP COLUMN IF EXISTS public_key,
   DROP COLUMN IF EXISTS private_key;
