ALTER TABLE local_user
    ALTER COLUMN password_encrypted DROP NOT NULL;

CREATE TABLE oauth_provider
(
    id                      serial PRIMARY KEY,
    display_name            text                                   NOT NULL,
    auto_verify_email       boolean                  DEFAULT TRUE  NOT NULL,
    account_linking_enabled boolean                  DEFAULT FALSE NOT NULL,
    enabled                 boolean                  DEFAULT TRUE  NOT NULL,
    published_at               timestamp with time zone DEFAULT now() NOT NULL,
    updated_at                 timestamp with time zone
);

CREATE UNIQUE INDEX oauth_provider_display_name_idx ON oauth_provider(display_name);

INSERT INTO oauth_provider (id, display_name, auto_verify_email,
                                   account_linking_enabled, enabled, published_at, updated_at)
VALUES (1, 'google', true, true, true,
        '2025-07-12 06:09:55.267522 +00:00', '2025-07-12 13:07:26.043000 +00:00');

ALTER TABLE local_site
    ADD COLUMN oauth_registration boolean DEFAULT FALSE NOT NULL;

CREATE TABLE oauth_account
(
    local_user_id     int REFERENCES local_user ON UPDATE CASCADE ON DELETE CASCADE      NOT NULL,
    oauth_provider_id int REFERENCES oauth_provider ON UPDATE CASCADE ON DELETE RESTRICT NOT NULL,
    oauth_user_id     text                                                               NOT NULL,
    published         timestamp with time zone DEFAULT now()                             NOT NULL,
    updated           timestamp with time zone,
    UNIQUE (oauth_provider_id, oauth_user_id),
    PRIMARY KEY (oauth_provider_id, local_user_id)
);

