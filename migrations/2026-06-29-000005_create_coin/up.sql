CREATE TABLE public.coin (
    id integer NOT NULL,
    code text NOT NULL,
    name text NOT NULL,
    supply_total integer DEFAULT 0 NOT NULL,
    supply_minted_total integer DEFAULT 0 NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone
);

CREATE SEQUENCE public.coin_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.coin_id_seq OWNED BY public.coin.id;

ALTER TABLE ONLY public.coin ALTER COLUMN id SET DEFAULT nextval('public.coin_id_seq'::regclass);

ALTER TABLE ONLY public.coin
    ADD CONSTRAINT coin_code_key UNIQUE (code);

ALTER TABLE ONLY public.coin
    ADD CONSTRAINT coin_pkey PRIMARY KEY (id);

CREATE INDEX idx_coin_code ON public.coin USING btree (code);
