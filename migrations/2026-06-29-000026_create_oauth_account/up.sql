CREATE TABLE public.oauth_account (
    local_user_id integer NOT NULL,
    oauth_provider_id integer NOT NULL,
    provider_account_id text NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone
);

ALTER TABLE ONLY public.oauth_account
    ADD CONSTRAINT oauth_account_oauth_provider_id_provider_account_id_key UNIQUE (oauth_provider_id, provider_account_id);

ALTER TABLE ONLY public.oauth_account
    ADD CONSTRAINT oauth_account_pkey PRIMARY KEY (oauth_provider_id, local_user_id);
