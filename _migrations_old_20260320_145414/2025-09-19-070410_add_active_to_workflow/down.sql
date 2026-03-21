DROP INDEX IF EXISTS ux_workflow_room_active_once;
ALTER TABLE workflow DROP COLUMN IF EXISTS active;