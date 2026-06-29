CREATE TABLE public.person_post_mention (
    id integer NOT NULL,
    recipient_id integer NOT NULL,
    post_id integer NOT NULL,
    read boolean DEFAULT false NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE SEQUENCE public.person_post_mention_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.person_post_mention_id_seq OWNED BY public.person_post_mention.id;

ALTER TABLE ONLY public.person_post_mention ALTER COLUMN id SET DEFAULT nextval('public.person_post_mention_id_seq'::regclass);

ALTER TABLE ONLY public.person_post_mention
    ADD CONSTRAINT person_post_mention_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.person_post_mention
    ADD CONSTRAINT person_post_mention_unique UNIQUE (recipient_id, post_id);
