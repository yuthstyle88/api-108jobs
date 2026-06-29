CREATE TABLE public.currency (
    id integer NOT NULL,
    code character varying(3) NOT NULL,
    name character varying(50) NOT NULL,
    symbol character varying(10) NOT NULL,
    numeric_code integer NOT NULL,
    coin_to_currency_rate integer DEFAULT 1 NOT NULL,
    decimal_places integer DEFAULT 2 NOT NULL,
    thousands_separator character varying(1) DEFAULT ','::character varying,
    decimal_separator character varying(1) DEFAULT '.'::character varying,
    symbol_position character varying(10) DEFAULT 'prefix'::character varying,
    is_active boolean DEFAULT true,
    is_default boolean DEFAULT false,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone,
    rate_last_updated_at timestamp with time zone,
    rate_last_updated_by integer
);

CREATE SEQUENCE public.currency_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.currency_id_seq OWNED BY public.currency.id;

ALTER TABLE ONLY public.currency ALTER COLUMN id SET DEFAULT nextval('public.currency_id_seq'::regclass);

ALTER TABLE ONLY public.currency
    ADD CONSTRAINT currency_code_key UNIQUE (code);

ALTER TABLE ONLY public.currency
    ADD CONSTRAINT currency_numeric_code_key UNIQUE (numeric_code);

ALTER TABLE ONLY public.currency
    ADD CONSTRAINT currency_pkey PRIMARY KEY (id);

CREATE INDEX idx_currency_numeric_code ON public.currency USING btree (numeric_code);
