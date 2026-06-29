CREATE TABLE public.user_review (
    id integer NOT NULL,
    reviewer_id integer NOT NULL,
    reviewee_id integer NOT NULL,
    workflow_id integer NOT NULL,
    rating smallint NOT NULL,
    comment text,
    created_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone,
    CONSTRAINT user_review_rating_check CHECK (((rating >= 1) AND (rating <= 5)))
);

CREATE SEQUENCE public.user_review_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.user_review_id_seq OWNED BY public.user_review.id;

ALTER TABLE ONLY public.user_review ALTER COLUMN id SET DEFAULT nextval('public.user_review_id_seq'::regclass);

ALTER TABLE ONLY public.user_review
    ADD CONSTRAINT uq_user_review_per_workflow UNIQUE (reviewer_id, reviewee_id, workflow_id);

ALTER TABLE ONLY public.user_review
    ADD CONSTRAINT user_review_pkey PRIMARY KEY (id);

CREATE INDEX idx_user_review_reviewee_id ON public.user_review USING btree (reviewee_id);

CREATE INDEX idx_user_review_workflow_id ON public.user_review USING btree (workflow_id);
