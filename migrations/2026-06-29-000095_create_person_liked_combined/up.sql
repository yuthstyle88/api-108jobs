CREATE TABLE public.person_liked_combined (
    id integer NOT NULL,
    liked_at timestamp with time zone NOT NULL,
    like_score smallint NOT NULL,
    person_id integer NOT NULL,
    post_id integer,
    proposal_id integer,
    CONSTRAINT person_liked_combined_check CHECK ((num_nonnulls(post_id, proposal_id) = 1))
);

CREATE SEQUENCE public.person_liked_combined_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.person_liked_combined_id_seq OWNED BY public.person_liked_combined.id;

ALTER TABLE ONLY public.person_liked_combined ALTER COLUMN id SET DEFAULT nextval('public.person_liked_combined_id_seq'::regclass);

ALTER TABLE ONLY public.person_liked_combined
    ADD CONSTRAINT person_liked_combined_person_id_post_id_key UNIQUE (person_id, post_id);

ALTER TABLE ONLY public.person_liked_combined
    ADD CONSTRAINT person_liked_combined_person_id_proposal_id_key UNIQUE (person_id, proposal_id);

ALTER TABLE ONLY public.person_liked_combined
    ADD CONSTRAINT person_liked_combined_pkey PRIMARY KEY (id);

CREATE INDEX idx_person_liked_combined ON public.person_liked_combined USING btree (person_id);

CREATE INDEX idx_person_liked_combined_published ON public.person_liked_combined USING btree (liked_at DESC, id DESC);
