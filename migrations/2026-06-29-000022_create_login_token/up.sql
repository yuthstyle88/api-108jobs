CREATE TABLE public.login_token (
    token text NOT NULL,
    user_id integer NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    ip text,
    user_agent text
);

ALTER TABLE ONLY public.login_token
    ADD CONSTRAINT login_token_pkey PRIMARY KEY (token);

CREATE INDEX idx_login_token_user_token ON public.login_token USING btree (user_id, token);
