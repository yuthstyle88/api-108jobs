-- Remove the wallet transaction tracking columns
ALTER TABLE delivery_details DROP COLUMN IF EXISTS rider_wallet_transaction_id;
ALTER TABLE delivery_details DROP COLUMN IF EXISTS employer_wallet_transaction_id;
ALTER TABLE delivery_details DROP COLUMN IF EXISTS employer_confirmed_at;

-- Remove the delivery_fee column
ALTER TABLE delivery_details DROP COLUMN IF EXISTS delivery_fee;
