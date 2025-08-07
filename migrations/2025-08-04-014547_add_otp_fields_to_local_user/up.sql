-- Add OTP fields to local_user table and additional fields to person table
DO $$
BEGIN
    -- Add OTP fields to local_user if they don't exist
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'local_user' AND column_name = 'otp_secret') THEN
        ALTER TABLE local_user ADD COLUMN otp_secret text;
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'local_user' AND column_name = 'otp_url') THEN
        ALTER TABLE local_user ADD COLUMN otp_url text;
    END IF;
    
    -- Add wallet_id to person table if it doesn't exist
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'person' AND column_name = 'wallet_id') THEN
        ALTER TABLE person ADD COLUMN wallet_id INTEGER REFERENCES wallet(id) ON DELETE SET NULL;
    END IF;
    
    -- Add public_key and private_key to person table if they don't exist
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'person' AND column_name = 'public_key') THEN
        ALTER TABLE person ADD COLUMN public_key text;
    END IF;
    
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'person' AND column_name = 'private_key') THEN
        ALTER TABLE person ADD COLUMN private_key text;
    END IF;
END
$$;
