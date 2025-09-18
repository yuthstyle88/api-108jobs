DROP INDEX IF EXISTS idx_chat_room_current_comment_id;
ALTER TABLE chat_room
    DROP COLUMN IF EXISTS current_comment_id;
