ALTER TABLE workflow
    ADD COLUMN room_id varchar REFERENCES chat_room(id) ON UPDATE CASCADE ON DELETE CASCADE;

CREATE INDEX idx_workflow_room_id ON workflow(room_id);