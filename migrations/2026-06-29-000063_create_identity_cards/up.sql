CREATE TABLE public.identity_cards (
    id integer NOT NULL,
    address_id integer,
    id_number character varying(64) NOT NULL,
    issued_date date NOT NULL,
    expiry_date date NOT NULL,
    full_name character varying(255) NOT NULL,
    date_of_birth date NOT NULL,
    nationality character varying(255) NOT NULL,
    is_verified boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone
);

CREATE SEQUENCE public.identity_cards_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.identity_cards_id_seq OWNED BY public.identity_cards.id;

ALTER TABLE ONLY public.identity_cards ALTER COLUMN id SET DEFAULT nextval('public.identity_cards_id_seq'::regclass);

ALTER TABLE ONLY public.identity_cards
    ADD CONSTRAINT identity_cards_pkey PRIMARY KEY (id);

CREATE INDEX idx_identity_cards_verified ON public.identity_cards USING btree (is_verified);
