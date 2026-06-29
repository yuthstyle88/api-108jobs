CREATE TABLE public.proposal_reply (
    id integer NOT NULL,
    recipient_id integer NOT NULL,
    comment_id integer NOT NULL,
    read boolean DEFAULT false NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE SEQUENCE public.proposal_reply_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.proposal_reply_id_seq OWNED BY public.proposal_reply.id;

ALTER TABLE ONLY public.proposal_reply ALTER COLUMN id SET DEFAULT nextval('public.proposal_reply_id_seq'::regclass);

ALTER TABLE ONLY public.proposal_reply
    ADD CONSTRAINT proposal_reply_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.proposal_reply
    ADD CONSTRAINT proposal_reply_recipient_id_proposal_id_key UNIQUE (recipient_id, comment_id);

CREATE INDEX idx_proposal_reply_proposal ON public.proposal_reply USING btree (comment_id);

CREATE INDEX idx_proposal_reply_published ON public.proposal_reply USING btree (published_at DESC);

CREATE INDEX idx_proposal_reply_recipient ON public.proposal_reply USING btree (recipient_id);
