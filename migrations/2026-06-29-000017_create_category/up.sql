CREATE TABLE public.category (
    id integer NOT NULL,
    name character varying(255) NOT NULL,
    title character varying(255) NOT NULL,
    sidebar text,
    removed boolean DEFAULT false NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone,
    deleted boolean DEFAULT false NOT NULL,
    self_promotion boolean DEFAULT false NOT NULL,
    last_refreshed_at timestamp with time zone DEFAULT now() NOT NULL,
    icon text,
    banner text,
    posting_restricted_to_mods boolean DEFAULT false NOT NULL,
    instance_id integer,
    visibility public.category_visibility DEFAULT 'Public'::public.category_visibility NOT NULL,
    description character varying(150),
    random_number smallint DEFAULT public.random_smallint() NOT NULL,
    subscribers bigint DEFAULT 0 NOT NULL,
    posts bigint DEFAULT 0 NOT NULL,
    proposals bigint DEFAULT 0 NOT NULL,
    users_active_day bigint DEFAULT 0 NOT NULL,
    users_active_week bigint DEFAULT 0 NOT NULL,
    users_active_month bigint DEFAULT 0 NOT NULL,
    users_active_half_year bigint DEFAULT 0 NOT NULL,
    hot_rank double precision DEFAULT 0.0001 NOT NULL,
    subscribers_local bigint DEFAULT 0 NOT NULL,
    report_count smallint DEFAULT 0 NOT NULL,
    unresolved_report_count smallint DEFAULT 0 NOT NULL,
    interactions_month bigint DEFAULT 0 NOT NULL,
    local_removed boolean DEFAULT false NOT NULL,
    path public.ltree DEFAULT '0'::public.ltree NOT NULL,
    active boolean DEFAULT true NOT NULL,
    is_new boolean DEFAULT true NOT NULL
);

CREATE SEQUENCE public.category_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.category_id_seq OWNED BY public.category.id;

ALTER TABLE ONLY public.category ALTER COLUMN id SET DEFAULT nextval('public.category_id_seq'::regclass);

ALTER TABLE ONLY public.category
    ADD CONSTRAINT category_pkey PRIMARY KEY (id);

CREATE INDEX idx_category_hot ON public.category USING btree (hot_rank DESC, id DESC);

CREATE INDEX idx_category_lower_name ON public.category USING btree (lower((name)::text) DESC, id DESC);

CREATE INDEX idx_category_nonzero_hotrank ON public.category USING btree (published_at) WHERE (hot_rank <> (0)::double precision);

CREATE INDEX idx_category_path ON public.category USING gist (path);

CREATE INDEX idx_category_posts ON public.category USING btree (posts DESC, id DESC);

CREATE INDEX idx_category_proposals ON public.category USING btree (proposals DESC, id DESC);

CREATE INDEX idx_category_published ON public.category USING btree (published_at DESC, id DESC);

CREATE INDEX idx_category_subscribers ON public.category USING btree (subscribers DESC, id DESC);

CREATE INDEX idx_category_subscribers_local ON public.category USING btree (subscribers_local DESC, id DESC);

CREATE INDEX idx_category_title ON public.category USING btree (title DESC, id DESC);

CREATE INDEX idx_category_trigram ON public.category USING gin (name public.gin_trgm_ops, title public.gin_trgm_ops);

CREATE INDEX idx_category_users_active_day ON public.category USING btree (users_active_day DESC, id DESC);

CREATE INDEX idx_category_users_active_half_year ON public.category USING btree (users_active_half_year DESC, id DESC);

CREATE INDEX idx_category_users_active_month ON public.category USING btree (users_active_month DESC, id DESC);

CREATE INDEX idx_category_users_active_week ON public.category USING btree (users_active_week DESC, id DESC);
