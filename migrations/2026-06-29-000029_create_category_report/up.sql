CREATE TABLE public.category_report (
    id integer NOT NULL,
    creator_id integer NOT NULL,
    category_id integer NOT NULL,
    original_category_name text NOT NULL,
    original_category_title text NOT NULL,
    original_category_description text,
    original_category_sidebar text,
    original_category_icon text,
    original_category_banner text,
    reason text NOT NULL,
    resolved boolean DEFAULT false NOT NULL,
    resolver_id integer,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone
);

CREATE SEQUENCE public.category_report_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.category_report_id_seq OWNED BY public.category_report.id;

ALTER TABLE ONLY public.category_report ALTER COLUMN id SET DEFAULT nextval('public.category_report_id_seq'::regclass);

ALTER TABLE ONLY public.category_report
    ADD CONSTRAINT category_report_category_id_creator_id_key UNIQUE (category_id, creator_id);

ALTER TABLE ONLY public.category_report
    ADD CONSTRAINT category_report_pkey PRIMARY KEY (id);

CREATE INDEX idx_category_report_published ON public.category_report USING btree (published_at DESC);
