CREATE TABLE public.admin_allow_instance (
    id integer NOT NULL,
    instance_id integer NOT NULL,
    admin_person_id integer NOT NULL,
    allowed boolean NOT NULL,
    reason text,
    published_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE SEQUENCE public.admin_allow_instance_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.admin_allow_instance_id_seq OWNED BY public.admin_allow_instance.id;

ALTER TABLE ONLY public.admin_allow_instance ALTER COLUMN id SET DEFAULT nextval('public.admin_allow_instance_id_seq'::regclass);

ALTER TABLE ONLY public.admin_allow_instance
    ADD CONSTRAINT admin_allow_instance_pkey PRIMARY KEY (id);
