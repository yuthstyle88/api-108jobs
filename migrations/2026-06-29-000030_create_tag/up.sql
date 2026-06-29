CREATE TABLE public.tag (
    id integer NOT NULL,
    ap_id text DEFAULT public.generate_unique_changeme() NOT NULL,
    display_name text NOT NULL,
    category_id integer NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone,
    deleted boolean DEFAULT false NOT NULL
);

CREATE SEQUENCE public.tag_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.tag_id_seq OWNED BY public.tag.id;

ALTER TABLE ONLY public.tag ALTER COLUMN id SET DEFAULT nextval('public.tag_id_seq'::regclass);

ALTER TABLE ONLY public.tag
    ADD CONSTRAINT tag_ap_id_key UNIQUE (ap_id);

ALTER TABLE ONLY public.tag
    ADD CONSTRAINT tag_pkey PRIMARY KEY (id);
