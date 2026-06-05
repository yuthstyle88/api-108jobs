-- Add assignment tracking fields to delivery_details
ALTER TABLE delivery_details
ADD COLUMN assigned_rider_id INT REFERENCES rider(id) ON DELETE SET NULL,
ADD COLUMN assigned_at TIMESTAMPTZ,
ADD COLUMN assigned_by_person_id INT REFERENCES person(id) ON DELETE SET NULL,
ADD COLUMN linked_comment_id INT REFERENCES comment(id) ON DELETE SET NULL;

-- Index for querying rider's active deliveries
CREATE INDEX idx_delivery_details_assigned_rider ON delivery_details(assigned_rider_id)
WHERE status = 'Assigned';
