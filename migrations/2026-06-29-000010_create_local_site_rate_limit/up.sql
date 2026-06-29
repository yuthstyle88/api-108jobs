CREATE TABLE public.local_site_rate_limit (
    local_site_id integer NOT NULL,
    message_max_requests integer DEFAULT 180 NOT NULL,
    message_interval_seconds integer DEFAULT 60 NOT NULL,
    post_max_requests integer DEFAULT 6 NOT NULL,
    post_interval_seconds integer DEFAULT 600 NOT NULL,
    register_max_requests integer DEFAULT 10 NOT NULL,
    register_interval_seconds integer DEFAULT 3600 NOT NULL,
    image_max_requests integer DEFAULT 6 NOT NULL,
    image_interval_seconds integer DEFAULT 3600 NOT NULL,
    proposal_max_requests integer DEFAULT 6 NOT NULL,
    proposal_interval_seconds integer DEFAULT 600 NOT NULL,
    search_max_requests integer DEFAULT 60 NOT NULL,
    search_interval_seconds integer DEFAULT 600 NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone,
    import_user_settings_max_requests integer DEFAULT 1 NOT NULL,
    import_user_settings_interval_seconds integer DEFAULT 86400 NOT NULL
);

ALTER TABLE ONLY public.local_site_rate_limit
    ADD CONSTRAINT local_site_rate_limit_pkey PRIMARY KEY (local_site_id);
