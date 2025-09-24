ALTER TABLE billing
    ADD COLUMN room_id varchar REFERENCES chat_room(id) ON UPDATE CASCADE ON DELETE CASCADE;

CREATE INDEX idx_billing_room_id ON billing(room_id);