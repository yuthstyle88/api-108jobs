-- down.sql — extra defensive version

DROP INDEX IF EXISTS idx_delivery_details_status;
DROP INDEX IF EXISTS idx_delivery_details_post_id;

DROP TABLE IF EXISTS delivery_details;

-- Only attempt to drop delivery_status if no columns reference it anymore
DO $$
DECLARE
usage_count integer;
BEGIN
SELECT COUNT(*)
INTO usage_count
FROM information_schema.columns
WHERE data_type = 'USER-DEFINED'
  AND udt_name = 'delivery_status';

IF usage_count = 0 THEN
DROP TYPE IF EXISTS delivery_status;
RAISE NOTICE 'delivery_status enum dropped (no remaining columns use it)';
ELSE
    RAISE NOTICE 'delivery_status enum kept (still used in % columns)', usage_count;
END IF;
END $$;