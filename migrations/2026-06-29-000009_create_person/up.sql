CREATE TABLE public.person (
    id integer NOT NULL,
    name character varying(255) NOT NULL,
    display_name character varying(255),
    avatar text,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone,
    bio text,
    last_refreshed_at timestamp with time zone DEFAULT now() NOT NULL,
    banner text,
    deleted boolean DEFAULT false NOT NULL,
    matrix_user_id text,
    bot_account boolean DEFAULT false NOT NULL,
    instance_id integer NOT NULL,
    post_count bigint DEFAULT 0 NOT NULL,
    post_score bigint DEFAULT 0 NOT NULL,
    proposal_count bigint DEFAULT 0 NOT NULL,
    proposal_score bigint DEFAULT 0 NOT NULL,
    wallet_id integer NOT NULL,
    shared_key text,
    private_key text,
    contacts text,
    skills text,
    work_samples jsonb,
    portfolio_pics jsonb,
    available boolean DEFAULT true NOT NULL,
    is_secure_message boolean DEFAULT false NOT NULL
);

CREATE SEQUENCE public.person_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.person_id_seq OWNED BY public.person.id;

ALTER TABLE ONLY public.person ALTER COLUMN id SET DEFAULT nextval('public.person_id_seq'::regclass);

ALTER TABLE ONLY public.person
    ADD CONSTRAINT person__pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.person
    ADD CONSTRAINT person_wallet_id_key UNIQUE (wallet_id);

CREATE INDEX idx_person_lower_name ON public.person USING btree (lower((name)::text));

CREATE UNIQUE INDEX idx_person_lower_name_unique ON public.person USING btree (lower((name)::text));

CREATE INDEX idx_person_portfolio_pics ON public.person USING gin (portfolio_pics);

CREATE INDEX idx_person_published ON public.person USING btree (published_at DESC);

CREATE INDEX idx_person_trigram ON public.person USING gin (name public.gin_trgm_ops, display_name public.gin_trgm_ops);

CREATE INDEX idx_person_wallet_id ON public.person USING btree (wallet_id);

CREATE INDEX idx_person_work_samples ON public.person USING gin (work_samples);
