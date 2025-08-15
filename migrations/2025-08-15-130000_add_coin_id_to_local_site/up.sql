-- Add coin_id to local_site table to specify which coin the platform uses
ALTER TABLE local_site ADD COLUMN coin_id INT4 REFERENCES coin(id);

-- Add index for better query performance
CREATE INDEX local_site_coin_id_idx ON local_site(coin_id);