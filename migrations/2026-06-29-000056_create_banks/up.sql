CREATE TABLE public.banks (
    id integer NOT NULL,
    name character varying(255) NOT NULL,
    country_id character(2) NOT NULL,
    bank_code character varying(20),
    swift_code character varying(20),
    is_active boolean DEFAULT true,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone,
    CONSTRAINT chk_country_id_format CHECK ((country_id ~ '^[A-Z]{2}$'::text)),
    CONSTRAINT chk_swift_format CHECK (((swift_code IS NULL) OR ((swift_code)::text ~ '^[A-Z0-9]{8}([A-Z0-9]{3})?$'::text)))
);

CREATE SEQUENCE public.banks_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.banks_id_seq OWNED BY public.banks.id;

ALTER TABLE ONLY public.banks ALTER COLUMN id SET DEFAULT nextval('public.banks_id_seq'::regclass);

ALTER TABLE ONLY public.banks
    ADD CONSTRAINT banks_pkey PRIMARY KEY (id);

CREATE INDEX idx_banks_country ON public.banks USING btree (country_id);
