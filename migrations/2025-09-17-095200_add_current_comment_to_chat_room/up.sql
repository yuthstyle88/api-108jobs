ALTER TABLE chat_room
    ADD COLUMN IF NOT EXISTS current_comment_id int REFERENCES comment(id) ON UPDATE CASCADE ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS idx_chat_room_current_comment_id ON chat_room(current_comment_id);
