-- Add delivery_fee column to delivery_details table
-- This stores the agreed fee amount that will be held in escrow when a rider is assigned
ALTER TABLE delivery_details ADD COLUMN delivery_fee INTEGER NOT NULL DEFAULT 0;

-- Add employer_confirmed_at timestamp to track when employer confirmed completion
ALTER TABLE delivery_details ADD COLUMN employer_confirmed_at TIMESTAMPTZ;

-- Add employer_wallet_transaction_id to track the escrow hold transaction
ALTER TABLE delivery_details ADD COLUMN employer_wallet_transaction_id INT;

-- Add rider_wallet_transaction_id to track the payment release transaction
ALTER TABLE delivery_details ADD COLUMN rider_wallet_transaction_id INT;
