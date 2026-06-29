CREATE TABLE public.admin_purge_proposal (
    id integer NOT NULL,
    admin_person_id integer NOT NULL,
    post_id integer NOT NULL,
    reason text,
    published_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE SEQUENCE public.admin_purge_proposal_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.admin_purge_proposal_id_seq OWNED BY public.admin_purge_proposal.id;

ALTER TABLE ONLY public.admin_purge_proposal ALTER COLUMN id SET DEFAULT nextval('public.admin_purge_proposal_id_seq'::regclass);

ALTER TABLE ONLY public.admin_purge_proposal
    ADD CONSTRAINT admin_purge_proposal_pkey PRIMARY KEY (id);
