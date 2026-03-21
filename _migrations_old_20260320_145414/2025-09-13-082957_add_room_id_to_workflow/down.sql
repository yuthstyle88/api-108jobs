ALTER TABLE chat_room
DROP COLUMN IF EXISTS post_id;

DROP INDEX IF EXISTS idx_workflow_room_id;

ALTER TABLE workflow
DROP COLUMN IF EXISTS room_id;