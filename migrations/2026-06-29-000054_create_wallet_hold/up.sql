CREATE TABLE public.wallet_hold (
    id bigint NOT NULL,
    wallet_id integer NOT NULL,
    billing_id integer NOT NULL,
    amount integer NOT NULL,
    status text NOT NULL,
    idempotency_key text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    released_at timestamp with time zone,
    CONSTRAINT wallet_hold_amount_check CHECK ((amount > 0)),
    CONSTRAINT wallet_hold_status_check CHECK ((status = ANY (ARRAY['Active'::text, 'Released'::text, 'Captured'::text])))
);

CREATE SEQUENCE public.wallet_hold_id_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.wallet_hold_id_seq OWNED BY public.wallet_hold.id;

ALTER TABLE ONLY public.wallet_hold ALTER COLUMN id SET DEFAULT nextval('public.wallet_hold_id_seq'::regclass);

ALTER TABLE ONLY public.wallet_hold
    ADD CONSTRAINT wallet_hold_pkey PRIMARY KEY (id);

CREATE INDEX idx_wallet_hold_billing ON public.wallet_hold USING btree (billing_id);

CREATE INDEX idx_wallet_hold_status ON public.wallet_hold USING btree (status);

CREATE INDEX idx_wallet_hold_wallet ON public.wallet_hold USING btree (wallet_id);

CREATE UNIQUE INDEX uq_wallet_hold_active_per_billing ON public.wallet_hold USING btree (billing_id) WHERE (status = 'Active'::text);

CREATE UNIQUE INDEX uq_wallet_hold_idem ON public.wallet_hold USING btree (idempotency_key) WHERE (idempotency_key IS NOT NULL);
