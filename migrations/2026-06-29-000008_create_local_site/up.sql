CREATE TABLE public.local_site (
    id integer NOT NULL,
    site_id integer NOT NULL,
    site_setup boolean DEFAULT false NOT NULL,
    enable_self_promotion boolean DEFAULT true NOT NULL,
    category_creation_admin_only boolean DEFAULT false NOT NULL,
    require_email_verification boolean DEFAULT false NOT NULL,
    application_question text DEFAULT 'to verify that you are human, please explain why you want to create an account on this site'::text,
    private_instance boolean DEFAULT false NOT NULL,
    default_theme text DEFAULT 'browser'::text NOT NULL,
    default_post_listing_type public.listing_type_enum DEFAULT 'Local'::public.listing_type_enum NOT NULL,
    legal_information text,
    application_email_admins boolean DEFAULT false NOT NULL,
    slur_filter_regex text,
    actor_name_max_length integer DEFAULT 20 NOT NULL,
    captcha_enabled boolean DEFAULT false NOT NULL,
    captcha_difficulty character varying(255) DEFAULT 'medium'::character varying NOT NULL,
    published_at timestamp with time zone DEFAULT now() NOT NULL,
    updated_at timestamp with time zone,
    registration_mode public.registration_mode_enum DEFAULT 'Open'::public.registration_mode_enum NOT NULL,
    reports_email_admins boolean DEFAULT false NOT NULL,
    federation_signed_fetch boolean DEFAULT false NOT NULL,
    default_post_listing_mode public.post_listing_mode_enum DEFAULT 'List'::public.post_listing_mode_enum NOT NULL,
    default_post_sort_type public.post_sort_type_enum DEFAULT 'Active'::public.post_sort_type_enum NOT NULL,
    default_proposal_sort_type public.proposal_sort_type_enum DEFAULT 'Hot'::public.proposal_sort_type_enum NOT NULL,
    oauth_registration boolean DEFAULT false NOT NULL,
    post_upvotes public.federation_mode_enum DEFAULT 'All'::public.federation_mode_enum NOT NULL,
    post_downvotes public.federation_mode_enum DEFAULT 'All'::public.federation_mode_enum NOT NULL,
    comment_upvotes public.federation_mode_enum DEFAULT 'All'::public.federation_mode_enum NOT NULL,
    comment_downvotes public.federation_mode_enum DEFAULT 'All'::public.federation_mode_enum NOT NULL,
    default_post_time_range_seconds integer,
    disallow_self_promotion_content boolean DEFAULT false NOT NULL,
    users bigint DEFAULT 1 NOT NULL,
    posts bigint DEFAULT 0 NOT NULL,
    proposals bigint DEFAULT 0 NOT NULL,
    communities bigint DEFAULT 0 NOT NULL,
    users_active_day bigint DEFAULT 0 NOT NULL,
    users_active_week bigint DEFAULT 0 NOT NULL,
    users_active_month bigint DEFAULT 0 NOT NULL,
    users_active_half_year bigint DEFAULT 0 NOT NULL,
    disable_email_notifications boolean DEFAULT false NOT NULL,
    verify_with_otp boolean DEFAULT true NOT NULL,
    coin_id integer
);

CREATE SEQUENCE public.local_site_id_seq
    AS integer
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE public.local_site_id_seq OWNED BY public.local_site.id;

ALTER TABLE ONLY public.local_site ALTER COLUMN id SET DEFAULT nextval('public.local_site_id_seq'::regclass);

ALTER TABLE ONLY public.local_site
    ADD CONSTRAINT local_site_pkey PRIMARY KEY (id);

ALTER TABLE ONLY public.local_site
    ADD CONSTRAINT local_site_site_id_key UNIQUE (site_id);

CREATE INDEX local_site_coin_id_idx ON public.local_site USING btree (coin_id);
