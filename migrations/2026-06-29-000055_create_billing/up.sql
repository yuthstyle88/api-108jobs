CREATE TABLE public.billing (
    id integer NOT NULL,
    freelancer_id integer NOT NULL,
    employer_id integer NOT NULL,
    post_id integer NOT NULL,
    proposal_id integer,
    amount integer NOT NULL,
    description text NOT NULL,
    status public.billing_status DEFAULT 'QuotePendingReview'::public.billing_status NOT NULL,
    work_description text,
    deliverable_url text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone,
    paid_at timestamp with time zone,
    room_id character varying,
    CONSTRAINT billing_amount_check CHECK ((amount > 0))
);

CREATE SEQUENCE public.billing_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.billing_id_seq OWNED BY public.billing.id;

ALTER TABLE ONLY public.billing ALTER COLUMN id SET DEFAULT nextval('public.billing_id_seq'::regclass);

ALTER TABLE ONLY public.billing
    ADD CONSTRAINT billing_pkey PRIMARY KEY (id);

CREATE INDEX idx_billing_created_at ON public.billing USING btree (created_at DESC);

CREATE INDEX idx_billing_employer_id ON public.billing USING btree (employer_id);

CREATE INDEX idx_billing_freelancer_id ON public.billing USING btree (freelancer_id);

CREATE INDEX idx_billing_post_id ON public.billing USING btree (post_id);

CREATE INDEX idx_billing_room_id ON public.billing USING btree (room_id);
