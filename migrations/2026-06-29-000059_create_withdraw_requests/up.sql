CREATE TABLE public.withdraw_requests (
    id integer NOT NULL,
    local_user_id integer NOT NULL,
    wallet_id integer NOT NULL,
    user_bank_account_id integer NOT NULL,
    amount integer DEFAULT 0 NOT NULL,
    status public.withdraw_status DEFAULT 'Pending'::public.withdraw_status NOT NULL,
    reason text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    currency_id integer DEFAULT 1 NOT NULL,
    amount_currency double precision DEFAULT 0 NOT NULL,
    conversion_rate_used integer DEFAULT 1 NOT NULL
);

CREATE SEQUENCE public.withdraw_requests_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.withdraw_requests_id_seq OWNED BY public.withdraw_requests.id;

ALTER TABLE ONLY public.withdraw_requests ALTER COLUMN id SET DEFAULT nextval('public.withdraw_requests_id_seq'::regclass);

ALTER TABLE ONLY public.withdraw_requests
    ADD CONSTRAINT withdraw_requests_pkey PRIMARY KEY (id);

CREATE INDEX idx_withdraw_requests_currency ON public.withdraw_requests USING btree (currency_id);
