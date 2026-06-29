CREATE TABLE public.proposal_report (
    id integer NOT NULL,
    creator_id integer NOT NULL,
    comment_id integer NOT NULL,
    original_comment_text text NOT NULL,
    reason text NOT NULL,
    resolved boolean DEFAULT false NOT NULL,
    resolver_id integer,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone,
    violates_instance_rules boolean DEFAULT false NOT NULL
);

CREATE SEQUENCE public.proposal_report_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.proposal_report_id_seq OWNED BY public.proposal_report.id;

ALTER TABLE ONLY public.proposal_report ALTER COLUMN id SET DEFAULT nextval('public.proposal_report_id_seq'::regclass);

ALTER TABLE ONLY public.proposal_report
    ADD CONSTRAINT proposal_report_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.proposal_report
    ADD CONSTRAINT proposal_report_proposal_id_creator_id_key UNIQUE (comment_id, creator_id);

CREATE INDEX idx_proposal_report_published ON public.proposal_report USING btree (published_at DESC);
