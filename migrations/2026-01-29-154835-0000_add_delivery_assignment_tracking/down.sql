-- Remove assignment tracking fields from delivery_details
ALTER TABLE delivery_details DROP COLUMN IF EXISTS linked_comment_id;
ALTER TABLE delivery_details DROP COLUMN IF EXISTS assigned_by_person_id;
ALTER TABLE delivery_details DROP COLUMN IF EXISTS assigned_at;
ALTER TABLE delivery_details DROP COLUMN IF EXISTS assigned_rider_id;

-- Remove the index
DROP INDEX IF EXISTS idx_delivery_details_assigned_rider;
