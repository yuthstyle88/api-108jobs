ALTER TABLE workflow
    ADD COLUMN room_id varchar REFERENCES chat_room(id) ON UPDATE CASCADE ON DELETE CASCADE;

CREATE INDEX idx_workflow_room_id ON workflow(room_id);

ALTER TABLE chat_room
    ADD COLUMN IF NOT EXISTS post_id int REFERENCES post(id);