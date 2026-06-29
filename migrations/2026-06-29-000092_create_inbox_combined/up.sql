CREATE TABLE public.inbox_combined (
    id integer NOT NULL,
    published_at timestamp with time zone NOT NULL,
    proposal_reply_id integer,
    person_proposal_mention_id integer,
    person_post_mention_id integer,
    CONSTRAINT inbox_combined_check CHECK ((num_nonnulls(proposal_reply_id, person_proposal_mention_id, person_post_mention_id) = 1))
);

CREATE SEQUENCE public.inbox_combined_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.inbox_combined_id_seq OWNED BY public.inbox_combined.id;

ALTER TABLE ONLY public.inbox_combined ALTER COLUMN id SET DEFAULT nextval('public.inbox_combined_id_seq'::regclass);

ALTER TABLE ONLY public.inbox_combined
    ADD CONSTRAINT inbox_combined_person_post_mention_id_key UNIQUE (person_post_mention_id);

ALTER TABLE ONLY public.inbox_combined
    ADD CONSTRAINT inbox_combined_person_proposal_mention_id_key UNIQUE (person_proposal_mention_id);

ALTER TABLE ONLY public.inbox_combined
    ADD CONSTRAINT inbox_combined_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.inbox_combined
    ADD CONSTRAINT inbox_combined_proposal_reply_id_key UNIQUE (proposal_reply_id);

CREATE INDEX idx_inbox_combined_published ON public.inbox_combined USING btree (published_at DESC, id DESC);

CREATE INDEX idx_inbox_combined_published_asc ON public.inbox_combined USING btree (public.reverse_timestamp_sort(published_at) DESC, id DESC);
