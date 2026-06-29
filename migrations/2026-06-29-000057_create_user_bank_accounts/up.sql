CREATE TABLE public.user_bank_accounts (
    id integer NOT NULL,
    local_user_id integer NOT NULL,
    bank_id integer NOT NULL,
    account_number character varying(50) NOT NULL,
    account_name character varying(255) NOT NULL,
    is_default boolean DEFAULT false NOT NULL,
    is_verified boolean DEFAULT false NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone,
    verification_image_path character varying(255)
);

CREATE SEQUENCE public.user_bank_accounts_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.user_bank_accounts_id_seq OWNED BY public.user_bank_accounts.id;

ALTER TABLE ONLY public.user_bank_accounts ALTER COLUMN id SET DEFAULT nextval('public.user_bank_accounts_id_seq'::regclass);

ALTER TABLE ONLY public.user_bank_accounts
    ADD CONSTRAINT user_bank_accounts_pkey PRIMARY KEY (id);

CREATE INDEX idx_user_bank_accounts_bank_id ON public.user_bank_accounts USING btree (bank_id);

CREATE INDEX idx_user_bank_accounts_default ON public.user_bank_accounts USING btree (local_user_id, is_default) WHERE (is_default = true);

CREATE INDEX idx_user_bank_accounts_local_user_id ON public.user_bank_accounts USING btree (local_user_id);

CREATE UNIQUE INDEX uniq_default_bank_account_per_user ON public.user_bank_accounts USING btree (local_user_id) WHERE (is_default = true);

CREATE UNIQUE INDEX uniq_user_bank_account ON public.user_bank_accounts USING btree (local_user_id, bank_id, account_number);
