CREATE TABLE public.person_content_combined (
    id integer NOT NULL,
    published_at timestamp with time zone NOT NULL,
    post_id integer,
    proposal_id integer,
    CONSTRAINT person_content_combined_check CHECK ((num_nonnulls(post_id, proposal_id) = 1))
);

CREATE SEQUENCE public.person_content_combined_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.person_content_combined_id_seq OWNED BY public.person_content_combined.id;

ALTER TABLE ONLY public.person_content_combined ALTER COLUMN id SET DEFAULT nextval('public.person_content_combined_id_seq'::regclass);

ALTER TABLE ONLY public.person_content_combined
    ADD CONSTRAINT person_content_combined_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.person_content_combined
    ADD CONSTRAINT person_content_combined_post_id_key UNIQUE (post_id);

ALTER TABLE ONLY public.person_content_combined
    ADD CONSTRAINT person_content_combined_proposal_id_key UNIQUE (proposal_id);

CREATE INDEX idx_person_content_combined_published ON public.person_content_combined USING btree (published_at DESC, id DESC);
