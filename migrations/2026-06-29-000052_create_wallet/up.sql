CREATE TABLE public.wallet (
    id integer NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone,
    is_platform boolean DEFAULT false NOT NULL,
    balance_total integer DEFAULT 0 NOT NULL,
    balance_available integer DEFAULT 0 NOT NULL,
    balance_outstanding integer DEFAULT 0 NOT NULL,
    version bigint DEFAULT 0 NOT NULL
);

CREATE SEQUENCE public.wallet_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.wallet_id_seq OWNED BY public.wallet.id;

ALTER TABLE ONLY public.wallet ALTER COLUMN id SET DEFAULT nextval('public.wallet_id_seq'::regclass);

ALTER TABLE ONLY public.wallet
    ADD CONSTRAINT wallet_pkey PRIMARY KEY (id);

CREATE UNIQUE INDEX idx_wallet_platform_singleton ON public.wallet USING btree (is_platform) WHERE (is_platform = true);
