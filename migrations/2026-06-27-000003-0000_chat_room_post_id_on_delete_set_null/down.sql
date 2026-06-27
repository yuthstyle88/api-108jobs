-- Restore the original constraint (no explicit ON DELETE action).
ALTER TABLE chat_room
    DROP CONSTRAINT IF EXISTS chat_room_post_id_fkey;

ALTER TABLE chat_room
    ADD CONSTRAINT chat_room_post_id_fkey FOREIGN KEY (post_id) REFERENCES post (id);
