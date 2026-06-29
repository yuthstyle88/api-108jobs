CREATE TABLE public.post (
    id integer NOT NULL,
    name character varying(200) NOT NULL,
    url character varying(2000),
    body text,
    creator_id integer NOT NULL,
    category_id integer,
    removed boolean DEFAULT false NOT NULL,
    locked boolean DEFAULT false NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone,
    deleted boolean DEFAULT false NOT NULL,
    self_promotion boolean DEFAULT false NOT NULL,
    embed_title text,
    embed_description text,
    thumbnail_url text,
    embed_video_url text,
    language_id integer DEFAULT 0 NOT NULL,
    featured_category boolean DEFAULT false NOT NULL,
    featured_local boolean DEFAULT false NOT NULL,
    url_content_type text,
    alt_text text,
    scheduled_publish_time_at timestamp with time zone,
    proposals bigint DEFAULT 0 NOT NULL,
    score bigint DEFAULT 0 NOT NULL,
    upvotes bigint DEFAULT 0 NOT NULL,
    downvotes bigint DEFAULT 0 NOT NULL,
    newest_proposal_time_necro_at timestamp with time zone DEFAULT now() NOT NULL,
    newest_proposal_time_at timestamp with time zone DEFAULT now() NOT NULL,
    hot_rank double precision DEFAULT 0.0001 NOT NULL,
    hot_rank_active double precision DEFAULT 0.0001 NOT NULL,
    controversy_rank double precision DEFAULT 0 NOT NULL,
    scaled_rank double precision DEFAULT 0.0001 NOT NULL,
    report_count smallint DEFAULT 0 NOT NULL,
    unresolved_report_count smallint DEFAULT 0 NOT NULL,
    federation_pending boolean DEFAULT false NOT NULL,
    pending boolean DEFAULT false NOT NULL,
    is_english_required boolean DEFAULT false NOT NULL,
    budget integer DEFAULT 0 NOT NULL,
    deadline timestamp with time zone,
    job_type public.job_type_enum DEFAULT 'PartTime'::public.job_type_enum NOT NULL,
    intended_use public.intended_use_enum DEFAULT 'Unknown'::public.intended_use_enum NOT NULL,
    post_kind public.post_kind DEFAULT 'Normal'::public.post_kind NOT NULL
);

CREATE SEQUENCE public.post_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.post_id_seq OWNED BY public.post.id;

ALTER TABLE ONLY public.post ALTER COLUMN id SET DEFAULT nextval('public.post_id_seq'::regclass);

ALTER TABLE ONLY public.post
    ADD CONSTRAINT post_pkey PRIMARY KEY (id);

CREATE INDEX idx_post_category ON public.post USING btree (category_id);

CREATE INDEX idx_post_category_active ON public.post USING btree (category_id, featured_local DESC, hot_rank_active DESC, published_at DESC, id DESC);

CREATE INDEX idx_post_category_controversy ON public.post USING btree (category_id, featured_local DESC, controversy_rank DESC, id DESC);

CREATE INDEX idx_post_category_hot ON public.post USING btree (category_id, featured_local DESC, hot_rank DESC, published_at DESC, id DESC);

CREATE INDEX idx_post_category_most_proposals ON public.post USING btree (category_id, featured_local DESC, proposals DESC, published_at DESC, id DESC);

CREATE INDEX idx_post_category_newest_proposal_time ON public.post USING btree (category_id, featured_local DESC, newest_proposal_time_at DESC, id DESC);

CREATE INDEX idx_post_category_newest_proposal_time_necro ON public.post USING btree (category_id, featured_local DESC, newest_proposal_time_necro_at DESC, id DESC);

CREATE INDEX idx_post_category_published ON public.post USING btree (category_id, published_at DESC, id DESC);

CREATE INDEX idx_post_category_scaled ON public.post USING btree (category_id, featured_local DESC, scaled_rank DESC, published_at DESC, id DESC);

CREATE INDEX idx_post_category_score ON public.post USING btree (category_id, featured_local DESC, score DESC, published_at DESC, id DESC);

CREATE INDEX idx_post_creator ON public.post USING btree (creator_id);

CREATE INDEX idx_post_featured_category_active ON public.post USING btree (category_id, featured_category DESC, hot_rank_active DESC, published_at DESC, id DESC);

CREATE INDEX idx_post_featured_category_controversy ON public.post USING btree (category_id, featured_category DESC, controversy_rank DESC, id DESC);

CREATE INDEX idx_post_featured_category_hot ON public.post USING btree (category_id, featured_category DESC, hot_rank DESC, published_at DESC, id DESC);

CREATE INDEX idx_post_featured_category_most_proposals ON public.post USING btree (category_id, featured_category DESC, proposals DESC, published_at DESC, id DESC);

CREATE INDEX idx_post_featured_category_newest_proposal_time ON public.post USING btree (category_id, featured_category DESC, newest_proposal_time_at DESC, id DESC);

CREATE INDEX idx_post_featured_category_newest_proposal_time_necr ON public.post USING btree (category_id, featured_category DESC, newest_proposal_time_necro_at DESC, id DESC);

CREATE INDEX idx_post_featured_category_published ON public.post USING btree (category_id, featured_category DESC, published_at DESC, id DESC);

CREATE INDEX idx_post_featured_category_scaled ON public.post USING btree (category_id, featured_category DESC, scaled_rank DESC, published_at DESC, id DESC);

CREATE INDEX idx_post_featured_category_score ON public.post USING btree (category_id, featured_category DESC, score DESC, published_at DESC, id DESC);

CREATE INDEX idx_post_featured_local_active ON public.post USING btree (featured_local DESC, hot_rank_active DESC, published_at DESC, id DESC);

CREATE INDEX idx_post_featured_local_controversy ON public.post USING btree (featured_local DESC, controversy_rank DESC, id DESC);

CREATE INDEX idx_post_featured_local_hot ON public.post USING btree (featured_local DESC, hot_rank DESC, published_at DESC, id DESC);

CREATE INDEX idx_post_featured_local_most_proposals ON public.post USING btree (featured_local DESC, proposals DESC, published_at DESC, id DESC);

CREATE INDEX idx_post_featured_local_newest_proposal_time ON public.post USING btree (featured_local DESC, newest_proposal_time_at DESC, id DESC);

CREATE INDEX idx_post_featured_local_newest_proposal_time_necro ON public.post USING btree (featured_local DESC, newest_proposal_time_necro_at DESC, id DESC);

CREATE INDEX idx_post_featured_local_published ON public.post USING btree (featured_local DESC, published_at DESC, id DESC);

CREATE INDEX idx_post_featured_local_scaled ON public.post USING btree (featured_local DESC, scaled_rank DESC, published_at DESC, id DESC);

CREATE INDEX idx_post_featured_local_score ON public.post USING btree (featured_local DESC, score DESC, published_at DESC, id DESC);

CREATE INDEX idx_post_language ON public.post USING btree (language_id);

CREATE INDEX idx_post_nonzero_hotrank ON public.post USING btree (published_at DESC) WHERE ((hot_rank <> (0)::double precision) OR (hot_rank_active <> (0)::double precision));

CREATE INDEX idx_post_published ON public.post USING btree (published_at DESC);

CREATE INDEX idx_post_scheduled_publish_time ON public.post USING btree (scheduled_publish_time_at);

CREATE INDEX idx_post_trigram ON public.post USING gin (name public.gin_trgm_ops, body public.gin_trgm_ops, alt_text public.gin_trgm_ops);

CREATE INDEX idx_post_url ON public.post USING btree (url);

CREATE INDEX idx_post_url_content_type ON public.post USING gin (url_content_type public.gin_trgm_ops);
