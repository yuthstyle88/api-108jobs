CREATE TABLE public.currency_rate_history (
    id integer NOT NULL,
    currency_id integer NOT NULL,
    old_rate integer NOT NULL,
    new_rate integer NOT NULL,
    changed_by integer,
    changed_at timestamp with time zone DEFAULT now() NOT NULL,
    reason text,
    created_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE SEQUENCE public.currency_rate_history_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.currency_rate_history_id_seq OWNED BY public.currency_rate_history.id;

ALTER TABLE ONLY public.currency_rate_history ALTER COLUMN id SET DEFAULT nextval('public.currency_rate_history_id_seq'::regclass);

ALTER TABLE ONLY public.currency_rate_history
    ADD CONSTRAINT currency_rate_history_pkey PRIMARY KEY (id);

CREATE INDEX idx_currency_rate_history_currency_date ON public.currency_rate_history USING btree (currency_id, changed_at DESC);
