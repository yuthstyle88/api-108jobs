-- Drop indexes
DROP INDEX IF EXISTS idx_delivery_rider_rating_employer_id;
DROP INDEX IF EXISTS idx_delivery_rider_rating_post_id;
DROP INDEX IF EXISTS idx_delivery_rider_rating_rider_id;

-- Drop table
DROP TABLE IF EXISTS delivery_rider_rating;
