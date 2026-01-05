ALTER TABLE chat_message
DROP COLUMN IF EXISTS sender_ack_confirmed_at;

DROP table IF EXISTS pending_sender_ack;