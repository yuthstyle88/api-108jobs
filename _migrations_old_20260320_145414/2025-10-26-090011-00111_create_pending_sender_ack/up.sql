-- ในตารางข้อความ เพิ่มคอลัมน์ไว้ track การยืนยัน ACK ของฝั่ง client
ALTER TABLE chat_message
    ADD COLUMN sender_ack_confirmed_at TIMESTAMPTZ;

-- เก็บ token ของ ACK ที่รอ client ยืนยัน (แยกตาราง จะค้น/เตือนง่าย)
CREATE TABLE pending_sender_ack (
                                    id BIGSERIAL PRIMARY KEY,
                                    room_id      TEXT NOT NULL,
                                    sender_id    INT NOT NULL,
                                    client_id    UUID NOT NULL,
                                    server_id    INT NOT NULL,
                                    created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- ดัชนีช่วยค้นตามผู้ส่ง/ห้อง
CREATE INDEX IF NOT EXISTS idx_pending_sender_ack_created_at ON pending_sender_ack (created_at);
CREATE INDEX IF NOT EXISTS idx_pending_sender_ack_stream ON pending_sender_ack (room_id, sender_id);