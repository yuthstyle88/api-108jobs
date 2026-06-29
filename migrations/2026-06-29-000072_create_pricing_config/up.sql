CREATE TABLE public.pricing_config (
    id integer NOT NULL,
    currency_id integer NOT NULL,
    name character varying(100) NOT NULL,
    base_fare_coin integer NOT NULL,
    time_charge_per_minute_coin integer NOT NULL,
    minimum_charge_minutes integer DEFAULT 10,
    distance_charge_per_km_coin integer NOT NULL,
    accepts_cash boolean DEFAULT true,
    accepts_coin boolean DEFAULT true,
    is_active boolean DEFAULT true,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone
);

CREATE SEQUENCE public.pricing_config_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.pricing_config_id_seq OWNED BY public.pricing_config.id;

ALTER TABLE ONLY public.pricing_config ALTER COLUMN id SET DEFAULT nextval('public.pricing_config_id_seq'::regclass);

ALTER TABLE ONLY public.pricing_config
    ADD CONSTRAINT pricing_config_pkey PRIMARY KEY (id);

CREATE INDEX idx_pricing_config_currency_active ON public.pricing_config USING btree (currency_id, is_active);
