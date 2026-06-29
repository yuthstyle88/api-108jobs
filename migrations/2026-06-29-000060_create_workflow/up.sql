CREATE TABLE public.workflow (
    id integer NOT NULL,
    post_id integer NOT NULL,
    seq_number smallint DEFAULT 1 NOT NULL,
    status public.workflow_status DEFAULT 'WaitForFreelancerQuotation'::public.workflow_status NOT NULL,
    revision_required boolean DEFAULT false NOT NULL,
    revision_count smallint DEFAULT 0 NOT NULL,
    revision_reason text,
    deliverable_version smallint DEFAULT 0 NOT NULL,
    deliverable_submitted_at timestamp with time zone,
    deliverable_accepted boolean DEFAULT false NOT NULL,
    accepted_at timestamp with time zone,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone,
    room_id character varying,
    deliverable_url text,
    active boolean DEFAULT true NOT NULL,
    status_before_cancel public.workflow_status,
    billing_id integer,
    CONSTRAINT workflow_seq_number_min_one CHECK ((seq_number >= 1))
);

CREATE SEQUENCE public.workflow_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.workflow_id_seq OWNED BY public.workflow.id;

ALTER TABLE ONLY public.workflow ALTER COLUMN id SET DEFAULT nextval('public.workflow_id_seq'::regclass);

ALTER TABLE ONLY public.workflow
    ADD CONSTRAINT workflow_pkey PRIMARY KEY (id);

CREATE INDEX idx_workflow_billing_id ON public.workflow USING btree (billing_id);

CREATE INDEX idx_workflow_post ON public.workflow USING btree (post_id);

CREATE INDEX idx_workflow_post_status ON public.workflow USING btree (post_id, status);

CREATE INDEX idx_workflow_room_id ON public.workflow USING btree (room_id);

CREATE UNIQUE INDEX ux_workflow_room_active_once ON public.workflow USING btree (room_id) WHERE (status <> ALL (ARRAY['Completed'::public.workflow_status, 'Cancelled'::public.workflow_status]));
