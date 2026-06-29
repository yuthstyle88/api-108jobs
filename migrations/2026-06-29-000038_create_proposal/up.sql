CREATE TABLE public.proposal (
    id integer NOT NULL,
    creator_id integer NOT NULL,
    post_id integer NOT NULL,
    content text NOT NULL,
    removed boolean DEFAULT false NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone,
    deleted boolean DEFAULT false NOT NULL,
    path public.ltree DEFAULT '0'::public.ltree NOT NULL,
    distinguished boolean DEFAULT false NOT NULL,
    language_id integer DEFAULT 0 NOT NULL,
    score bigint DEFAULT 0 NOT NULL,
    upvotes bigint DEFAULT 0 NOT NULL,
    downvotes bigint DEFAULT 0 NOT NULL,
    child_count integer DEFAULT 0 NOT NULL,
    hot_rank double precision DEFAULT 0.0001 NOT NULL,
    controversy_rank double precision DEFAULT 0 NOT NULL,
    report_count smallint DEFAULT 0 NOT NULL,
    unresolved_report_count smallint DEFAULT 0 NOT NULL,
    pending boolean DEFAULT true NOT NULL
);

CREATE SEQUENCE public.proposal_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.proposal_id_seq OWNED BY public.proposal.id;

ALTER TABLE ONLY public.proposal ALTER COLUMN id SET DEFAULT nextval('public.proposal_id_seq'::regclass);

ALTER TABLE ONLY public.proposal
    ADD CONSTRAINT proposal_pkey PRIMARY KEY (id);

CREATE INDEX idx_path_gist ON public.proposal USING gist (path);

CREATE INDEX idx_proposal_content_trigram ON public.proposal USING gin (content public.gin_trgm_ops);

CREATE INDEX idx_proposal_controversy ON public.proposal USING btree (controversy_rank DESC);

CREATE INDEX idx_proposal_creator ON public.proposal USING btree (creator_id);

CREATE INDEX idx_proposal_hot ON public.proposal USING btree (hot_rank DESC, score DESC);

CREATE INDEX idx_proposal_language ON public.proposal USING btree (language_id);

CREATE INDEX idx_proposal_nonzero_hotrank ON public.proposal USING btree (published_at) WHERE (hot_rank <> (0)::double precision);

CREATE INDEX idx_proposal_post ON public.proposal USING btree (post_id);

CREATE INDEX idx_proposal_published ON public.proposal USING btree (published_at DESC);

CREATE INDEX idx_proposal_score ON public.proposal USING btree (score DESC);
