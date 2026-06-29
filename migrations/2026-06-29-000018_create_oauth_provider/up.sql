CREATE TABLE public.oauth_provider (
    id integer NOT NULL,
    display_name text NOT NULL,
    issuer text NOT NULL,
    authorization_endpoint text NOT NULL,
    token_endpoint text NOT NULL,
    userinfo_endpoint text NOT NULL,
    id_claim text NOT NULL,
    client_id text NOT NULL,
    client_secret text NOT NULL,
    scopes text NOT NULL,
    auto_verify_email boolean DEFAULT true NOT NULL,
    account_linking_enabled boolean DEFAULT false NOT NULL,
    enabled boolean DEFAULT true NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone,
    use_pkce boolean DEFAULT false NOT NULL
);

CREATE SEQUENCE public.oauth_provider_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.oauth_provider_id_seq OWNED BY public.oauth_provider.id;

ALTER TABLE ONLY public.oauth_provider ALTER COLUMN id SET DEFAULT nextval('public.oauth_provider_id_seq'::regclass);

ALTER TABLE ONLY public.oauth_provider
    ADD CONSTRAINT oauth_provider_client_id_key UNIQUE (client_id);

ALTER TABLE ONLY public.oauth_provider
    ADD CONSTRAINT oauth_provider_pkey PRIMARY KEY (id);

CREATE UNIQUE INDEX oauth_provider_display_name_idx ON public.oauth_provider USING btree (display_name);
