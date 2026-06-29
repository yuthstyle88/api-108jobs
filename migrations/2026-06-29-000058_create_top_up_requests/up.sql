CREATE TABLE public.top_up_requests (
    id integer NOT NULL,
    local_user_id integer NOT NULL,
    amount double precision NOT NULL,
    qr_id text NOT NULL,
    cs_ext_expiry_time timestamp with time zone NOT NULL,
    status public.top_up_status DEFAULT 'Pending'::public.top_up_status NOT NULL,
    transferred boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    paid_at timestamp with time zone,
    currency_id integer DEFAULT 1 NOT NULL,
    amount_coin integer DEFAULT 0 NOT NULL,
    conversion_rate_used integer DEFAULT 1 NOT NULL,
    CONSTRAINT top_up_requests_amount_check CHECK ((amount > (0)::double precision))
);

CREATE SEQUENCE public.top_up_requests_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.top_up_requests_id_seq OWNED BY public.top_up_requests.id;

ALTER TABLE ONLY public.top_up_requests ALTER COLUMN id SET DEFAULT nextval('public.top_up_requests_id_seq'::regclass);

ALTER TABLE ONLY public.top_up_requests
    ADD CONSTRAINT top_up_requests_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.top_up_requests
    ADD CONSTRAINT top_up_requests_qr_id_key UNIQUE (qr_id);

ALTER TABLE ONLY public.top_up_requests
    ADD CONSTRAINT top_ups_unique_pair UNIQUE (local_user_id, qr_id);

CREATE INDEX idx_top_up_requests_created_at ON public.top_up_requests USING btree (created_at DESC);

CREATE INDEX idx_top_up_requests_currency ON public.top_up_requests USING btree (currency_id);

CREATE INDEX idx_top_up_requests_status ON public.top_up_requests USING btree (status);

CREATE INDEX idx_top_up_requests_user_id ON public.top_up_requests USING btree (local_user_id);
