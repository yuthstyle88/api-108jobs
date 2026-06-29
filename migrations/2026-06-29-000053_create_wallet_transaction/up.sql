CREATE TABLE public.wallet_transaction (
    id integer NOT NULL,
    wallet_id integer NOT NULL,
    reference_type text NOT NULL,
    reference_id integer NOT NULL,
    kind public.tx_kind DEFAULT 'Deposit'::public.tx_kind NOT NULL,
    amount integer NOT NULL,
    description text NOT NULL,
    counter_user_id integer,
    idempotency_key text NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT wallet_transaction_amount_check CHECK ((amount > 0))
);

CREATE SEQUENCE public.wallet_transaction_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.wallet_transaction_id_seq OWNED BY public.wallet_transaction.id;

ALTER TABLE ONLY public.wallet_transaction ALTER COLUMN id SET DEFAULT nextval('public.wallet_transaction_id_seq'::regclass);

ALTER TABLE ONLY public.wallet_transaction
    ADD CONSTRAINT wallet_transaction_pkey PRIMARY KEY (id);

CREATE UNIQUE INDEX idx_wallet_tx_idem ON public.wallet_transaction USING btree (idempotency_key, wallet_id);

CREATE INDEX idx_wallet_tx_ref_time ON public.wallet_transaction USING btree (reference_type, reference_id, created_at DESC);

CREATE INDEX idx_wallet_tx_wallet_time ON public.wallet_transaction USING btree (wallet_id, created_at DESC);
