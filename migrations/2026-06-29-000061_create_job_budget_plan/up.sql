CREATE TABLE public.job_budget_plan (
    id integer NOT NULL,
    post_id integer NOT NULL,
    total_amount integer NOT NULL,
    installments jsonb DEFAULT '[]'::jsonb NOT NULL,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone DEFAULT now() NOT NULL,
    CONSTRAINT job_budget_plan_installments_check CHECK ((jsonb_typeof(installments) = 'array'::text)),
    CONSTRAINT job_budget_plan_total_amount_check CHECK ((total_amount >= 0))
);

CREATE SEQUENCE public.job_budget_plan_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.job_budget_plan_id_seq OWNED BY public.job_budget_plan.id;

ALTER TABLE ONLY public.job_budget_plan ALTER COLUMN id SET DEFAULT nextval('public.job_budget_plan_id_seq'::regclass);

ALTER TABLE ONLY public.job_budget_plan
    ADD CONSTRAINT job_budget_plan_pkey PRIMARY KEY (id);

CREATE UNIQUE INDEX idx_job_budget_plan_post_unique ON public.job_budget_plan USING btree (post_id);
