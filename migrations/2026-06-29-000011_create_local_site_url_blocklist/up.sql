CREATE TABLE public.local_site_url_blocklist (
    id integer NOT NULL,
    url text NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone
);

CREATE SEQUENCE public.local_site_url_blocklist_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.local_site_url_blocklist_id_seq OWNED BY public.local_site_url_blocklist.id;

ALTER TABLE ONLY public.local_site_url_blocklist ALTER COLUMN id SET DEFAULT nextval('public.local_site_url_blocklist_id_seq'::regclass);

ALTER TABLE ONLY public.local_site_url_blocklist
    ADD CONSTRAINT local_site_url_blocklist_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.local_site_url_blocklist
    ADD CONSTRAINT local_site_url_blocklist_url_key UNIQUE (url);
