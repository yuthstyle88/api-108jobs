CREATE TABLE public.search_combined (
    id integer NOT NULL,
    published_at timestamp with time zone NOT NULL,
    score bigint DEFAULT 0 NOT NULL,
    post_id integer,
    proposal_id integer,
    category_id integer,
    person_id integer,
    CONSTRAINT search_combined_check CHECK ((num_nonnulls(post_id, proposal_id, category_id, person_id) = 1))
);

CREATE SEQUENCE public.search_combined_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.search_combined_id_seq OWNED BY public.search_combined.id;

ALTER TABLE ONLY public.search_combined ALTER COLUMN id SET DEFAULT nextval('public.search_combined_id_seq'::regclass);

ALTER TABLE ONLY public.search_combined
    ADD CONSTRAINT search_combined_category_id_key UNIQUE (category_id);

ALTER TABLE ONLY public.search_combined
    ADD CONSTRAINT search_combined_person_id_key UNIQUE (person_id);

ALTER TABLE ONLY public.search_combined
    ADD CONSTRAINT search_combined_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.search_combined
    ADD CONSTRAINT search_combined_post_id_key UNIQUE (post_id);

ALTER TABLE ONLY public.search_combined
    ADD CONSTRAINT search_combined_proposal_id_key UNIQUE (proposal_id);

CREATE INDEX idx_search_combined_published ON public.search_combined USING btree (published_at DESC, id DESC);

CREATE INDEX idx_search_combined_published_asc ON public.search_combined USING btree (public.reverse_timestamp_sort(published_at) DESC, id DESC);

CREATE INDEX idx_search_combined_score ON public.search_combined USING btree (score DESC, id DESC);
