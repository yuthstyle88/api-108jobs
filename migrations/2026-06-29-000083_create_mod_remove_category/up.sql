CREATE TABLE public.mod_remove_category (
    id integer NOT NULL,
    mod_person_id integer NOT NULL,
    category_id integer NOT NULL,
    reason text,
    removed boolean DEFAULT true NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE SEQUENCE public.mod_remove_category_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.mod_remove_category_id_seq OWNED BY public.mod_remove_category.id;

ALTER TABLE ONLY public.mod_remove_category ALTER COLUMN id SET DEFAULT nextval('public.mod_remove_category_id_seq'::regclass);

ALTER TABLE ONLY public.mod_remove_category
    ADD CONSTRAINT mod_remove_category_pkey PRIMARY KEY (id);
