-- Remove coin_id from local_site table
DROP INDEX local_site_coin_id_idx;
ALTER TABLE local_site DROP COLUMN coin_id;