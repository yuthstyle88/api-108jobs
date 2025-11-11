ALTER TABLE local_user
    ADD COLUMN secure_chat_enabled BOOL DEFAULT false NOT NULL;