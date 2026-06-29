CREATE TABLE public.mod_change_category_visibility (
    id integer NOT NULL,
    category_id integer NOT NULL,
    mod_person_id integer NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    visibility public.category_visibility NOT NULL
);

CREATE SEQUENCE public.mod_change_category_visibility_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.mod_change_category_visibility_id_seq OWNED BY public.mod_change_category_visibility.id;

ALTER TABLE ONLY public.mod_change_category_visibility ALTER COLUMN id SET DEFAULT nextval('public.mod_change_category_visibility_id_seq'::regclass);

ALTER TABLE ONLY public.mod_change_category_visibility
    ADD CONSTRAINT mod_change_category_visibility_pkey PRIMARY KEY (id);
