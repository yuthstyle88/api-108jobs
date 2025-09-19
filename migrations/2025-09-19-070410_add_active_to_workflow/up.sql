ALTER TABLE workflow ADD COLUMN IF NOT EXISTS active boolean NOT NULL DEFAULT true;

-- Ensure only one active workflow per room
CREATE UNIQUE INDEX IF NOT EXISTS ux_workflow_room_active_once
    ON workflow (room_id)
    WHERE active = true;