-- `chat_room.post_id` was added (migration 2025-09-13-082957_add_room_id_to_workflow)
-- as `int REFERENCES post(id)` with no ON DELETE action, so it defaults to
-- NO ACTION. When a post is deleted — directly, or via an `Instance::delete`
-- cascade in a test teardown — any chat_room that links to it blocks the
-- delete with `chat_room_post_id_fkey`. A failed teardown cascade then leaves
-- the whole instance's rows behind, contaminating later tests that count
-- admins/reports globally.
--
-- The sibling optional reference `current_comment_id` already uses
-- ON DELETE SET NULL (migration 2025-09-17-095200). Apply the same to
-- `post_id`: post_id is nullable, so dropping a post should null the link,
-- not block the delete. Recreates the constraint only; no data change.
ALTER TABLE chat_room
    DROP CONSTRAINT IF EXISTS chat_room_post_id_fkey;

ALTER TABLE chat_room
    ADD CONSTRAINT chat_room_post_id_fkey FOREIGN KEY (post_id) REFERENCES post (id) ON UPDATE CASCADE ON DELETE SET NULL;
