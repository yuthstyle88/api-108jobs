CREATE TABLE public.report_combined (
    id integer NOT NULL,
    published_at timestamp with time zone NOT NULL,
    post_report_id integer,
    proposal_report_id integer,
    category_report_id integer,
    CONSTRAINT report_combined_check CHECK ((num_nonnulls(post_report_id, proposal_report_id, category_report_id) = 1))
);

CREATE SEQUENCE public.report_combined_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.report_combined_id_seq OWNED BY public.report_combined.id;

ALTER TABLE ONLY public.report_combined ALTER COLUMN id SET DEFAULT nextval('public.report_combined_id_seq'::regclass);

ALTER TABLE ONLY public.report_combined
    ADD CONSTRAINT report_combined_category_report_id_key UNIQUE (category_report_id);

ALTER TABLE ONLY public.report_combined
    ADD CONSTRAINT report_combined_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.report_combined
    ADD CONSTRAINT report_combined_post_report_id_key UNIQUE (post_report_id);

ALTER TABLE ONLY public.report_combined
    ADD CONSTRAINT report_combined_proposal_report_id_key UNIQUE (proposal_report_id);

CREATE INDEX idx_report_combined_published ON public.report_combined USING btree (published_at DESC, id DESC);

CREATE INDEX idx_report_combined_published_asc ON public.report_combined USING btree (public.reverse_timestamp_sort(published_at) DESC, id DESC);
