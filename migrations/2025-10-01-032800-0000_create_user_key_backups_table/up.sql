CREATE TABLE user_key_backups
(
    id                    serial PRIMARY KEY,
    local_user_id         int        NOT NULL REFERENCES local_user
        ON UPDATE CASCADE ON DELETE CASCADE,
    encrypted_private_key bytea      NOT NULL, -- Encrypted PKCS#8 private key
    iv                    bytea      NOT NULL, -- Initialization vector
    salt                  bytea      NOT NULL, -- Salt for PBKDF2
    created_at            timestamptz NOT NULL DEFAULT now(),
    updated_at            timestamptz,
    UNIQUE (local_user_id)
);